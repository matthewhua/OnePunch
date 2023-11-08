package xyz.ariane.util.memodb

import akka.actor.ActorRef
import com.google.common.hash.Funnel
import org.slf4j.Logger
import xyz.ariane.util.concurrent.ActorCompletionStage
import xyz.ariane.util.dclog.lzDebug
import xyz.ariane.util.dclog.lzWarn
import xyz.ariane.util.json.generateJsonBytes4DB
import xyz.ariane.util.lang.profile
import xyz.ariane.util.lang.tryCatch
import xyz.ariane.util.memodb.ModificationTracker.CheckLevel
import xyz.ariane.util.memodb.ModificationTracker.Op
import xyz.ariane.util.memodb.ModificationTracker.Op.*
import xyz.ariane.util.memodbupgrade.AbstractVersionDbDataUpgrade
import xyz.ariane.util.monitor.TickEvent
import xyz.ariane.util.monitor.WdbStatsReporter
import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.util.*
import kotlin.collections.ArrayList
import kotlin.math.min

// 设置写数据库超时日志的统计显示模式
// 1：只统计计数，不显示具体类
// 其他：显示每一条超时日志。
@Volatile
var writeDbRetryCountLogMode = 1

/**
 * 只写内存数据库，负责跟踪所有持久化数据的变化，根据变化生成save，update，delete等数据库操作
 *
 */
open class WriteOnlyInMemoryDb4Json @JvmOverloads constructor(
    private val wdbType: Int,
    val parallelSave: Boolean,
    val parallelSaveGroup: Int,
    private val kryoUtil: KryoUtil,
    private val createACS: () -> ActorCompletionStage<Unit>,
    private val fetchDao: () -> CommonDao,
    private val wdbStatsReporter: WdbStatsReporter?,
    private val logger: Logger,
    private val esLogger: (Throwable, String) -> Unit,
    /** 每次tick最大检查 [ModificationTracker] 的个数，默认不限制 */
    /** 计算时间用的[Clock] */
    private val clock: Clock = Clock.systemDefaultZone(),
    /** [SyncOp]首次(非重试)提交监听器 */
    private val syncOpSubmissionListener: ((SyncOp) -> Unit)? = null,
    /** 每次批量执行[SyncOp]的数量 */
    private val batchSize: Int,
    /** 每次[tick]的期望最大耗时，实际的耗时可能略大于此值，默认为1ms */
    maxTickDuration: Duration = Duration.ofMillis(1L),
    val perfMonitorActor: ActorRef, // 数据落地监控用的Actor
    val hotspotStatsActor: ActorRef, // 序列化性能监控用的Actor
    var upgradeHandlers: List<AbstractVersionDbDataUpgrade>, // 数据库数据升级列表
) {

    /** 所有被跟踪的 [EntityWrapper] */
    // SS：跟踪查询表
    // SS：一个成熟的World服的ModificationTracker对象数量至少会在几十万到上百万级别！
    // SS：所以这里直接用HashMap是有性能问题的！
    internal val trackerMap: HashMap<EntityWrapperKey, ModificationTracker> = hashMapOf()

    /** 待检查变化的优先队列，按下次检查时间排优先级 */
    // SS：跟踪队列，根据下次检测时间排序
    internal val checkingQueue = PriorityQueue(
        Comparator
            .comparingLong<ModificationTracker> { it.nextCheckTime }
            .thenComparingInt { System.identityHashCode(it) }
            .thenComparingInt { it.entityWrapper.fetchPrimaryKey().hashCode() }
    )

    internal val dirtyCheck: Funnel<EntityWrapper<*>>
    internal val dirtyCheckSub: Funnel<Any>

    /** 此对象是否已停用 */
    private var stopped = false

    /** 已经提交的[SyncOp] */
    // 这些Op是在actor中运算出来的，所以属于逻辑操作。
    private val submittedSyncOpQueue = LinkedList<SyncOp>()

    private val maxTickDurationNanos = maxTickDuration.toNanos()

    // 每10秒的重试计数，如果这个数经常不为0，说明数据库压力很大！
    private var retryCount = 0
    private var countResetTime = System.nanoTime()

    init {
        dirtyCheck = Funnel { from, into ->
            // SS：使用kryo序列化为2进制数组。
            val kb = kryoUtil.serialize(from)

            // 存入Sink
            into.putBytes(kb.bytes, 0, kb.dataLength)
        }
        dirtyCheckSub = Funnel { from, into ->
            // SS：使用kryo序列化为2进制数组。
            val kb = kryoUtil.serialize(from)

            // 存入Sink
            into.putBytes(kb.bytes, 0, kb.dataLength)
        }
    }

    private fun checkState() {
        check(!stopped) { "Already stopped." }
    }

    private fun keyOf(ew: EntityWrapper<*>): EntityWrapperKey = EntityWrapperKey(ew)

    internal fun contains(ew: EntityWrapper<*>?): Boolean = trackerMap.containsKey(keyOf(ew as EntityWrapper<*>))

    /**
     * 合并新操作
     *
     * 这个只会被merge方法调用
     */
    private fun ModificationTracker.mergeNextOp(newOp: Op) {
        val ew = entityWrapper
        if (newOp == SAVE_OR_UPDATE ||
            newOp == ABORT_SAVE ||
            newOp == REPLACE
        ) {
            // SS：这3个操作是不允许直接执行的
            error("Explicit $newOp not supported. ew=$ew")
        }

        val next = nextOp
        if (next == null) {
            // SS：之前没有操作，直接使用当前的
            nextOp = newOp

        } else {
            nextOp = when (next) {
                SAVE -> when (newOp) {
                    SAVE -> error(duplicateOpErrorMsg(ew, next, newOp)) // SS：连续Save是不可能的！
                    DELETE -> ABORT_SAVE // SS：取消保存，清除
                    UPDATE -> SAVE // SS：维持Save
                    else -> error(impossibleNextOpErrorMsg(newOp)) // SS：其他3种操作是不允许的！
                }

                DELETE -> when (newOp) {
                    SAVE -> REPLACE // SS：替换，不过有这种可能吗？
                    DELETE -> error(duplicateOpErrorMsg(ew, next, newOp)) // SS：连续删除是不可能的
                    UPDATE -> error(conflictErrorMsg(ew, next, newOp)) // SS：删除后更新也是不可能的
                    else -> error(impossibleNextOpErrorMsg(newOp)) // SS：其他三种操作也是不可能的
                }

                UPDATE -> when (newOp) {
                    SAVE -> error(conflictErrorMsg(ew, next, newOp)) // SS：更新后保存是不可能的
                    DELETE -> DELETE // SS：更新为删除
                    UPDATE -> UPDATE // SS：维持Update
                    else -> error(impossibleNextOpErrorMsg(newOp)) // SS：其他三种操作是不可能的
                }

                SAVE_OR_UPDATE -> when (newOp) {
                    SAVE -> error(duplicateOpErrorMsg(ew, next, newOp)) // SS：连续Save或者Update后Save是不可能的！
                    DELETE -> DELETE // 这里delete可能失败(找不到对应的行)，但结果是一致的删除状态
                    UPDATE -> SAVE_OR_UPDATE // SS：维持
                    else -> error(impossibleNextOpErrorMsg(newOp))
                }

                else -> error(impossibleNextOpErrorMsg(next))
            }
        }

        logger.lzDebug { "Merge next ($next,$newOp)=>$nextOp, pending=$pendingOp" }
    }

    /**
     * 在用户提交写操作时合并pendingOp和nextOp，得到合成的nextOp
     */
    private fun ModificationTracker.mergeNextOpWithPendingOp() {
        val pending = pendingOp ?: return

        val next = nextOp ?: return

        val ew = entityWrapper
        nextOp = when (pending) {
            SAVE -> when (next) { // pending是Save
                SAVE -> error(duplicateOpErrorMsg(ew, pending, next))
                DELETE -> DELETE // next是Del，维持Del
                UPDATE -> UPDATE // next是Update，维持Update
                SAVE_OR_UPDATE -> UPDATE // 这里将next转为Update，因为pending正在执行Save
                ABORT_SAVE -> error(conflictErrorMsg(ew, pending, next))
                REPLACE -> REPLACE
            }

            DELETE -> when (next) { // SS：正在执行删除
                SAVE -> REPLACE
                DELETE -> null // 为保持最终状态为delete，这里直接清除nextOp，等待pending的delete操作成功执行即可
                UPDATE -> error(conflictErrorMsg(ew, pending, next))
                SAVE_OR_UPDATE -> SAVE
                ABORT_SAVE -> ABORT_SAVE
                REPLACE -> error(conflictErrorMsg(ew, pending, next))
            }

            UPDATE -> when (next) { // SS：正在执行更新
                SAVE -> error(conflictErrorMsg(ew, pending, next))
                DELETE -> DELETE
                UPDATE -> UPDATE
                SAVE_OR_UPDATE -> UPDATE
                ABORT_SAVE -> error(conflictErrorMsg(ew, pending, next))
                REPLACE -> REPLACE
            }

            SAVE_OR_UPDATE -> when (next) { // SS：正在执行报错或更新
                SAVE -> error(duplicateOpErrorMsg(ew, pending, next))
                DELETE -> DELETE
                UPDATE -> UPDATE
                SAVE_OR_UPDATE -> UPDATE
                ABORT_SAVE -> error(conflictErrorMsg(ew, pending, next))
                REPLACE -> REPLACE
            }

            else -> error(impossiblePendingOpErrorMsg(pending))
        }
        logger.lzDebug { "Merge pending on modify ($next,$pending)=>($nextOp,$pendingOp)" }
    }

    private fun conflictErrorMsg(ew: EntityWrapper<*>, curOp: Op, thenOp: Op) = "Conflict op $curOp then $thenOp, $ew"

    private fun duplicateOpErrorMsg(ew: EntityWrapper<*>, curOp: Op, thenOp: Op) =
        "Duplicate op $curOp and $thenOp. $ew"

    private fun impossibleNextOpErrorMsg(next: Op) = "Impossible next op: $next"

    private fun impossiblePendingOpErrorMsg(pending: Op) = "Impossible pending op: $pending"

    /**
     * 删除一个[EntityWrapper], open for mock test only
     *
     * @param ew     要删除的对象
     * @throws IllegalArgumentException 如果参数实力为null或者并不在此管理
     * @return tracker是否还存在
     */
    open fun delete(ew: EntityWrapper<*>): Boolean {
        checkState()

        // SS：找到tracker，然后发起一个Delete操作，如果之前有save操作，中止save
        val tracker =
            requireNotNull(trackerMap[keyOf(ew)]) { "Instance not found key=${ew.fetchPrimaryKey()}, class=${ew.javaClass}" }

        // 下面的做法需要斟酌下。
//        val tracker = trackerMap[keyOf(ew)]
//        if (tracker == null) {
//            // 找不到目标对象的tracker，重复执行Delete？
//            // 不再使用requireNotNull是为了 避免 某些情况下多次删除导致数据库的访问崩掉（比如心跳出问题了）！
//            logger.lzError { "Instance not found key=${ew.fetchPrimaryKey()}, class=${ew.javaClass}" }
//            return
//        }

        merge(tracker, DELETE)

        // ABORT_SAVE只在Delete操作中才会出现，所以下面才要判断，为true时删除tracker。
        if (tracker.nextOp == ABORT_SAVE) {
            logger.lzDebug { "Abort save. ${tracker.key}" }

            removeTracker(tracker)

            return false
        }

        return true
    }

    /*
     * 合并新操作到当前tracker中
     * 这会设置nextOp属性
     */
    private fun merge(tracker: ModificationTracker, op: Op) {
        logger.lzDebug { "$op ${tracker.key}" }

        val prevNextOp = tracker.nextOp
        try {
            // SS：和当前的nextOp合并
            tracker.mergeNextOp(op)

            // SS：和pending操作合并
            tracker.mergeNextOpWithPendingOp()

        } catch (e: Throwable) {
            // Merge should be atomic, rollback the next op
            // SS：出错的话，就回退到之前的操作
            tracker.nextOp = prevNextOp
            throw e
        }
    }

    /**
     * 创建新的[EntityWrapper],并准备insert到数据库中
     *
     * @param clazz  [EntityWrapper]实现类,需要有无参构造函数
     * @param entity 对应的持久化实体
     */
    open fun <EW : EntityWrapper<E>, E : IEntity> save(clazz: Class<EW>, entity: E): EW {
        checkState()

        return createWrapper({ clazz.getDeclaredConstructor().newInstance() as EW }, entity).apply { save(this) }
    }

    /**
     * 保存一个新的已经创建好的[EntityWrapper]，open for mock test only
     *
     * @param ew 创建好的实例
     */
    open fun <EW : EntityWrapper<E>, E : IEntity> save(ew: EW): EW {
        checkState()

        // SS：创建一个tracker或替代一个已经存在的tracker
        // SS：save是立即存库的！
        val newTracker = createTracker(ew, false)
        val existTracker = trackerMap[newTracker.key]
        if (existTracker == null) {
            // SS：发起一次Save操作
            newTracker.markAllDataDirty()
            merge(newTracker, SAVE)

            // SS：将tracker放入chekc队列
            putTracker(newTracker)

        } else {
            // SS：发起一次Save操作
            newTracker.markAllDataDirty()
            merge(existTracker, SAVE) //如果合并失败会抛出异常

            if (existTracker.nextOp == REPLACE) {
                checkingQueue.remove(existTracker)

                newTracker.pendingOp = existTracker.pendingOp
                newTracker.nextOp = SAVE_OR_UPDATE

                // 保留检查超时时间
                if (existTracker.hasPendingOp()) {
                    newTracker.nextCheckTime = existTracker.nextCheckTime
                }
                putTracker(newTracker)
            }
        }

        return ew
    }

    /**
     * 提交立即保存
     */
    open fun saveImmediately(ew: EntityWrapper<*>, now: Instant = clock.instant()) {
        val key = keyOf(ew)
        val tracker = trackerMap[key]
        if (tracker == null) {
            logger.error("Instance not found key=${ew.fetchPrimaryKey()}, class=${ew.javaClass}")
            return
        }
        val nowMilSec = now.toEpochMilli()

        if (tracker.nextCheckTime <= nowMilSec) {
            // 下次检测时间已经到了
            return
        }

        if (tracker.hasPendingOp()) {
            // 有正在存库的操作，打标记，不改检测时间
            tracker.needSaveImmediately = true
            return
        }

        // 修改检测时间为当前时间，重新入队列
        checkingQueue.remove(tracker)
        tracker.nextCheckTime = nowMilSec
        checkingQueue.add(tracker)
    }

//    /**
//     * 从entity恢复已经持久化的对象
//     *
//     * @param ewClazz [EntityWrapper]实现类
//     * @param entity  对应的持久化实体
//     */
//    fun <EW : EntityWrapper<E>, E : IEntity> recover(ewClazz: Class<EW>, entity: E): EW {
//        // SS：这个的创建方法是直接借助newInstance方法来创建实例
//        return recover(entity) { ewClazz.newInstance() }
//    }

    fun <EW : EntityWrapper<E>, E : IEntity> recover(entity: E, create: () -> EW): EW {
        return recover(createWrapper(create, entity))
    }

    fun <EW : EntityWrapper<E>, E : IEntity> recover(ew: EW): EW {
        checkState()

        // SS：创建tracker，理论上跟踪表中不可能存在这个tracker
        val tracker = createTracker(ew, false)
        require(!trackerMap.containsKey(tracker.key)) { "Duplicate entity: ${tracker.key}" }

        // SS：将tracker放入跟踪表中
        putTracker(tracker)

        return ew
    }

    private fun putTracker(tracker: ModificationTracker) {
        trackerMap[tracker.key] = tracker
        checkingQueue.offer(tracker)
    }

    // SS：借助创建方法和entity实例来创建wrapper
    private fun <EW : EntityWrapper<E>, E : IEntity> createWrapper(create: () -> EW, entity: E): EW {
        return create().apply {
            profile(false, perfMonitorActor, "wrap_" + entity.javaClass.name) {
                // SS：调用entity的wrap方法来初始化
                wrap(upgradeHandlers, entity)
            }
        }
    }

    /**
     * 创建Tracker，immediately表示是否立即检测脏
     */
    private fun createTracker(
        inst: EntityWrapper<*>,
        immediately: Boolean
    ): ModificationTracker {
        return ModificationTracker.createKryoTracker(
            hotspotStatsActor,
            inst,
            logger,
            clock.millis(),
            immediately,
            dirtyCheck,
            dirtyCheckSub
        )
    }

    @JvmOverloads
    fun tick(now: Instant = clock.instant()): DirtyCheckRt {
        val nanoStart = System.nanoTime()
        val nowMillis = now.toEpochMilli() // SS：当前毫秒

        val maxLoop = 65536
        val maxNanos = maxTickDurationNanos // 最大检测纳秒数
        var checkedNum = maxLoop
//        val delays = LinkedList<DelayInfo>()
        var delay3sNum = 0 // 超过3s的延迟数

        // SS：尝试一定次数
        var checkFinishRt = DirtyCheckResult.DIRTY_CHECK_OUT_LOOP
        for (i in 1..maxLoop) {
            val hasChecked = tickCheckNext(nowMillis) { key, delay ->
                // SS：这边的delay的意思是，将当前时间和tracker中的entity的下次检测时间相比较，看延迟了多久才检测的。
                // SS：如果延迟的太多，比如一个心跳周期以上，那么说明Check变更的压力有点大。
//                delays += DelayInfo(key.clazz.name, delay)
                if (delay > 3_000L) {
                    delay3sNum++
                }
            }

            // SS：两种情况下跳出循环，一种是没有需要check的了；另一种是超过最大check时间。
            if (!hasChecked) {
                checkedNum = i - 1
                checkFinishRt = DirtyCheckResult.DIRTY_CHECK_NO_MORE
                break
            }
            if (i and 0x0f == 0 && System.nanoTime() - nanoStart >= maxNanos) {
                checkedNum = i - 1
                checkFinishRt = DirtyCheckResult.DIRTY_CHECK_OUT_TIME
                break
            }
        }

        // SS：批量执行DB操作
        val submitNum = batchExecuteAllDbOp()

        val nanoEnd = System.nanoTime()
        val nanoCost = nanoEnd - nanoStart

        // SS：上报数据
        val dirtyCheckRt =
            DirtyCheckRt(wdbType, trackerMap.size, checkedNum, submitNum, checkFinishRt, delay3sNum, nanoCost)
        wdbStatsReporter?.report(TickEvent(dirtyCheckRt))

        return dirtyCheckRt
    }

    // SS：设置停止标记
    fun stop() {
        stopped = true
    }

    /**
     * 扫描所有对象变化并派发存库操作
     */
    @JvmOverloads
    fun forceCheckAll(now: Instant = clock.instant(), level: CheckLevel = CheckLevel.FULL) {
        checkState()

        val nowMillis = now.toEpochMilli()

        val poppedTrackers = LinkedList<ModificationTracker>()
        var mergeUpdateCount = 0
        var checkedNum = 0
        profile(false, perfMonitorActor, "force_check_all_$level") {
            val checkingQIter = checkingQueue.iterator()
            while (checkingQIter.hasNext()) {
                val tracker = checkingQIter.next()
                if (!tracker.hasPendingOp()) {
                    // 没有正在进行中的DB操作。
                    if (tracker.calcAndSubmitSyncOp(level)) {
                        checkingQIter.remove()
                        poppedTrackers.add(tracker)
                    }
                } else {
                    // 正在等待之前提交的存库操作完成，需要再次检查数据是否变化，否则可能导致回档
                    try {
                        if (!tracker.hasNextOp() && tracker.checkDirty(level)) {
                            merge(tracker, UPDATE)
                            ++mergeUpdateCount
                        }
                    } catch (e: Exception) {
                        // merge操作可能产生冲突，在此场景认为是正常情况
                        logger.lzDebug {
                            "Merge update abort, key=${tracker.key}, pendingOp=${tracker.pendingOp}, nextOp=${tracker.nextOp}, e=${e.javaClass}, msg=${e.message}"
                        }
                    }
                }
                checkedNum++
            }

            // 重新添加回待检测队列。
            poppedTrackers.forEach {
                it.setTimeoutCheck(nowMillis)
                checkingQueue.offer(it)
            }
        }

        val submitNum = batchExecuteAllDbOp()

        // SS：上报数据
        val r = wdbStatsReporter
        if (r != null) {
            val totalNum = trackerMap.size
            val checkFinishRt = DirtyCheckResult.DIRTY_CHECK_NO_MORE
            val dirtyCheckRt = DirtyCheckRt(wdbType, totalNum, checkedNum, submitNum, checkFinishRt, 0, 0L)
            r.report(TickEvent(dirtyCheckRt))
        }
    }

    /**
     * 检查当前是否还有没完成的操作，包括执行中的DB操作（pendingOp）和需要执行DB操作的（nextOp）
     */
    fun hasPendingNextSyncOps(): Boolean = trackerMap.values.any { it.isKnownDirty() }

    /**
     * 计算当前是没完成的操作的数量，包括执行中的DB操作（pendingOp）和需要执行DB操作的（nextOp）
     * 在大型的Actor中，trackerMap是极其大的，所以这个计数方法的消耗很大，不能随便调用！
     * @return Int
     */
    fun countPendingNextSyncOp(): Int = trackerMap.values.count { it.isKnownDirty() }

    /**
     * 计算当前是进行中的DB操作的数量，包括执行中的DB操作（pendingOp）
     * 在大型的Actor中，trackerMap是极其大的，所以这个计数方法的消耗很大，不能随便调用！
     */
    fun countPendingSyncOpNum(): Int = trackerMap.values.count { it.hasPendingOp() }

    fun showPendingSyncOp(num: Int): List<String> {
        val strList = ArrayList<String>(num)
        var dirtyNum = 0
        for (tracker in trackerMap.values) {
            if (!tracker.hasPendingOp()) {
                continue
            }

            val dirtyStr = tracker.entityWrapper::class.java.simpleName
            strList.add(dirtyStr)
            dirtyNum++
            if (dirtyNum >= num) {
                break
            }
        }

        return strList
    }

    /**
     * 统计跟踪数
     * @return Int
     */
    fun countTrackerNum(): Int = trackerMap.size

    private inline fun tickCheckNext(
        now: Long,
        handleResult: (key: EntityWrapperKey, delayMillis: Long) -> Unit
    ): Boolean {
        // SS：尝试弹出一个
        val tracker = checkingQueue.peek() ?: return false // O(1)

        val millisUntilNextCheck = tracker.nextCheckTime - now
        if (millisUntilNextCheck > 0L) {
            // SS：还没到时间
            return false
        }

        // SS：正式弹出一个
        checkingQueue.poll() // O(log(n))

        // 有进行中的Op就重新尝试进行中的Op
        // 没有就算一个Op
        // 再没有，就更新下下次检测时间。
        with(tracker) {
            when {
                !tracker.needSaveImmediately && hasPendingOp() -> {
                    // SS：有进行中的Op，需要重试了。
                    // timeout or failed
                    retryPendingOp() // SS：重试

                    setTimeoutCheck(now)
                }

                calcAndSubmitSyncOp(CheckLevel.MINOR) -> {
                    // 成功提交了一个操作
                    // new SyncOp submitted, check timeout next time
                    setTimeoutCheck(now)
                }

                else -> updateNextCheckTime(now) // SS：更新下次检测时间
            }
        }

        // O(log(n)) SS：重新压入队列
        checkingQueue.offer(tracker)

        handleResult(tracker.key, -millisUntilNextCheck)
        return true
    }

    /**
     * 考虑到数据库处理慢，主从切换时可能会丢失save等问题，重试时可以把SAVE和UPDATE变为幂等的SAVE_OR_UPDATE,可增加成功率
     */
    private fun ModificationTracker.retryPendingOp() {
        var op = requireNotNull(pendingOp)
        val orgOp = op

        // 如果是保存或更新操作，一律改成SaveOrUpdate
        if (op == SAVE || op == UPDATE) {
            op = SAVE_OR_UPDATE
            pendingOp = op
        }

        submitSyncOp(SyncOp(perfMonitorActor, this, op), this)

        if (writeDbRetryCountLogMode == 1) {
            // 下面的日志输出不那么准确，但能将日志间隔控制在10秒以上。
            val currentTime = System.nanoTime()
            val timeDiff = currentTime - countResetTime
            if (timeDiff > 10_000_000_000) {
                // 大于10秒，打印当前出错信息，并清空计数
                esLogger(
                    Throwable(),
                    "$orgOp of $key timeout, will retry. Op change($orgOp->$op), nextOp: $nextOp, timeout num: ${retryCount}"
                )
                countResetTime = currentTime
                retryCount = 0
            } else {
                retryCount++
            }

        } else {
            esLogger(Throwable(), "$orgOp of $key timeout, will retry. Op change($orgOp->$op), nextOp: $nextOp")
        }

    }

    /**
     * 计算并提交一个SyncOp
     *
     * 这中间会通过calcSyncOp生成一个nextOp，并在submit后，将nextOp转为pendingOp。
     * 因为这个方法只在tick和forceCheckAll中调用，所以只有这2个方法才能将nextOp转为pendingOp
     */
    private fun ModificationTracker.calcAndSubmitSyncOp(level: CheckLevel): Boolean {
        // SS：计算出一个SyncOp
        val syncOp = calcSyncOp(this, level) ?: return false

        // SS：提交
        submitSyncOp(syncOp, this)

        // next转pending
        // 因为发布了pendingOp，所以清空nextOp。
        pendingOp = syncOp.op
        nextOp = null

        // SS：这里调用自定义回调，可以用来执行一些观察
        syncOpSubmissionListener?.run {
            tryCatch(logger) { invoke(syncOp) }
        }
        return true
    }

    /*
     * 生成一个可能的SyncOp
     * 这个方法只会被自己或者tracker的calcAndSubmitSyncOp调用。
     */
    private fun calcSyncOp(tracker: ModificationTracker, level: CheckLevel): SyncOp? {
        val nextOp = tracker.nextOp
        return when (nextOp) {
            SAVE, DELETE, UPDATE, SAVE_OR_UPDATE -> {
                // SS：存在保存类的nextOp，直接发布一个SyncOp
                SyncOp(perfMonitorActor, tracker, nextOp)
            }

            null -> {
                // SS：不存在nextOp，计算是否有修改，有的话，发布一次Update，然后再重新计算。
                if (tracker.checkDirty(level)) {
                    // SS：因为tracker检测出来脏了，发布一次Update操作
                    // SS：nextOp会被设置（不可能是修改，nextOp为null才会执行到这里）
                    merge(tracker, UPDATE)

                    // SS：再次判断是否有新的同步操作
                    calcSyncOp(tracker, level)

                } else {
                    null
                }
            }

            else -> error("Unexpected next op: $nextOp")
        }
    }

    /**
     * 批量执行所有提交的DB操作
     *
     * 假设batch是100，队列中有999个Op，那么将会分成10批提交执行。
     */
    private fun batchExecuteAllDbOp(): Int {
        // SS：这个while会让执行分为多次，最多batchSize一次
        var submitNum = 0
        if (!parallelSave) {
            while (submittedSyncOpQueue.isNotEmpty()) {
                // SS：将提交队列中的请求一个个取出，放入batch列表中
                val batch = ArrayList<Pair<SyncOp, IEntity>>(Math.min(batchSize, submittedSyncOpQueue.size))
                for (i in 1..batchSize) {
                    val op: SyncOp? = submittedSyncOpQueue.poll()
                    if (op != null) {
                        val entity = op.entity
                        batch.add(Pair(op, entity))
                        submitNum++
                    } else {
                        break
                    }
                }

                // SS：执行一个批次的操作
                executeBatch(batch)
            }
        } else {
            while (submittedSyncOpQueue.isNotEmpty()) {
                // 准备用于批量提交的数组
                val batches = ArrayList<ArrayList<Pair<SyncOp, IEntity>>>(parallelSaveGroup)
                repeat(parallelSaveGroup) {
                    batches.add(ArrayList(min(batchSize, submittedSyncOpQueue.size)))
                }

                // SS：将提交队列中的请求一个个取出，放入batch列表中
                for (i in 1..batchSize) {
                    val op: SyncOp? = submittedSyncOpQueue.poll()
                    if (op != null) {
                        // 计算提交数组索引
                        val entity = op.entity
                        val index = if (entity is IParallelWrapEntity) {
                            entity.parallelIndex(parallelSaveGroup)
                        } else {
                            0
                        }

                        batches[index].add(Pair(op, entity))
                        submitNum++

                    } else {
                        break
                    }
                }

                // SS：执行一个批次的操作
                executeBatchParallel(batches)
            }
        }

        return submitNum
    }

    private fun executeBatch(batch: List<Pair<SyncOp, IEntity>>) {
        // SS：创建ACS执行异步请求
        createACS().supplyIoKt {
            // SS：IO在事务中执行
            fetchDao().execWithTransaction { session ->
                var entity: IEntity?
                try {
                    for (syncOp in batch) {
                        entity = syncOp.second
                        when (syncOp.first.op) {
                            SAVE -> session.save(entity)
                            DELETE -> session.delete(entity)
                            UPDATE -> session.update(entity)
                            SAVE_OR_UPDATE -> session.saveOrUpdate(entity)
                            else -> error("Unexpected op: $syncOp")
                        }
                    }
                } catch (e: Exception) {
                    throw e
                }
            }
        }.whenCompleteKt { _, err ->
            if (err == null) {
                tryCatch(logger) { batch.forEach { onSuccess(it.first) } }
            } else {
                esLogger(err, "执行DB操作时出现异常")
                tryCatch(logger) {
                    batch.forEach {
                        onFailure(it.first, err)
                    }
                }
            }
        }
    }

    private fun executeBatchParallel(batches: List<ArrayList<Pair<SyncOp, IEntity>>>) {
        // SS：创建ACS执行异步请求
        var batchIndex = 0
        batches.forEach { batch ->
            createACS().supplyIoKt("worldDbSave${batchIndex}") {
                // SS：IO在事务中执行
                fetchDao().execWithTransaction { session ->
                    var entity: IEntity?
                    try {
                        for (pair in batch) {
                            entity = pair.second
                            when (pair.first.op) {
                                SAVE -> session.save(entity)
                                DELETE -> session.delete(entity)
                                UPDATE -> session.update(entity)
                                SAVE_OR_UPDATE -> session.saveOrUpdate(entity)
                                else -> error("Unexpected op: ${pair.first.op}")
                            }
                        }
                    } catch (e: Exception) {
                        throw e
                    }
                }
            }.whenCompleteKt { _, err ->
                if (err == null) {
                    tryCatch(logger) { batch.forEach { onSuccess(it.first) } }
                } else {
                    esLogger(err, "执行DB操作时出现异常")
                    tryCatch(logger) {
                        batch.forEach {
                            onFailure(it.first, err)
                        }
                    }
                }
            }
            batchIndex++
        }

    }

    private fun submitSyncOp(syncOp: SyncOp, tracker: ModificationTracker) {
        // SS：压入提交队列
        submittedSyncOpQueue.offer(syncOp)

        if (syncOp.op == SAVE || syncOp.op == SAVE_OR_UPDATE) {
            // 新创建的ModificationTracker的内部hash值为初始状态。SAVE操作提交时可能已经发生了数据变化，
            // 因此需要同步hash值，否则可能会触发多余的UPDATE，
            // 还会在后续数据刚好再次被修改为初始状态时，导致无法发现数据改变，数据库一直保留次时提交的数据，造成数据回档。
            // SS
            // SS：这段刷新hash的代码放在这里而不是放在calcSyncOp中
            // SS：我想是因为retryPendingOp中也会调用这个方法，从而需要重新计算。
            tracker.cleanup()
        }

        debugLogSyncOp(syncOp, "submitted")
    }

    // SS：设置下次检测时间为超时时间，这通常是因为，这个方法只在有pendingOp进行中时才被调用。
    private fun ModificationTracker.setTimeoutCheck(now: Long) {
        nextCheckTime = now + operationTimeoutMillis
    }

    private fun debugLogSyncOp(syncOp: SyncOp, what: String) {
        logger.lzDebug { "$syncOp $what, qsize=${checkingQueue.size}" }
    }

    /**
     * 当存库操作成功时调用
     *
     * @param syncOp 操作
     */
    open fun onSuccess(syncOp: SyncOp) {
        debugLogSyncOp(syncOp, "succeeded")

        val tracker = trackerMap[syncOp.key]
        if (tracker == null) {
            esLogger(Throwable(), "$syncOp succeeded, but tracker not found!!")
            return
        }

        when {
            tracker.pendingOp == null -> {
                logger.lzDebug { "$syncOp succeeded and tracker already cleaned." }
                return
            }

            tracker.pendingOp == SAVE_OR_UPDATE && (syncOp.op == UPDATE || syncOp.op == SAVE) -> {
                logger.lzDebug { "$syncOp succeeded and pendingOp is transformed to ${tracker.pendingOp}" }
            }

            tracker.pendingOp == syncOp.op -> Unit
            else -> {
                logger.lzWarn { "$syncOp succeeded but pendingOp is ${tracker.pendingOp}" }
                return //这种情况可能是很久之前重试的操作成功了（例如SAVE_OR_UPDATE）但其对应的pendingOp早就被消费掉了，应该忽略掉此事件
            }
        }

        tracker.pendingOp = null

        if (syncOp.op == DELETE) {
            if (tracker.nextOp == null) {
                removeTracker(tracker)
                return
            } else {
                logger.lzDebug { "$syncOp succeeded and nextOp is ${tracker.nextOp}, keep the instance." }
            }
        }

        // 取消超时检查，恢复正常检查时间间隔
        checkingQueue.remove(tracker)
        // 需要立即存库的，将检测时间修改为当前
        if (tracker.needSaveImmediately) {
            tracker.nextCheckTime = clock.millis()
            tracker.needSaveImmediately = false
        } else {
            tracker.updateNextCheckTime(clock.millis())
        }
        checkingQueue.offer(tracker)
    }

    private fun removeTracker(tracker: ModificationTracker) {
        trackerMap.remove(tracker.key)
        checkingQueue.remove(tracker)

        logger.lzDebug { "Tracker removed, key=${tracker.key}" }
    }

    /**
     * 存库操作失败时调用，这里不进行立即重试，超时和失败统一按照超时逻辑重试，在[tickCheckNext]中进行
     * TODO 重试可能出现不可恢复的失败，考虑增加重试次数限制
     * @param syncOp 操作
     * @param err    异常
     */
    open fun onFailure(syncOp: SyncOp, err: Throwable?) {
        debugLogSyncOp(syncOp, "failed")
        if (err != null) {
            logger.error("${syncOp} has exception.", err)
        } else {
            logger.error("{} failed.", syncOp)
        }
        val tracker = trackerMap[syncOp.key] ?: if (syncOp.op == DELETE) {
            logger.lzDebug { "$syncOp failed and the tracker has been deleted." }
            return
        } else {
            error("$syncOp failed, but tracker not found!!")
        }
        when {
            //此次为重试失败，如save,delete，前面的操作应经成功并消费掉了pendingOp
            tracker.pendingOp == null -> logger.lzDebug {
                "$syncOp failed and tracker is already cleaned."
            }

            tracker.pendingOp == syncOp.op -> logger.lzDebug {
                "$syncOp failed, will retry on timeout."
            }

            tracker.pendingOp == SAVE_OR_UPDATE && (syncOp.op == UPDATE || syncOp.op == SAVE) -> logger.lzDebug {
                "$syncOp succeeded and pendingOp is transformed to ${tracker.pendingOp}"
            }

            else -> error("$syncOp failed but pendingOp is ${tracker.pendingOp}")
        }

        // 一旦存库失败，清除立即存库标记
        tracker.needSaveImmediately = false
    }

}
