package xyz.ariane.util.memodb

import akka.actor.ActorRef
import com.google.common.hash.Funnel
import com.google.common.hash.HashCode
import com.google.common.hash.HashFunction
import com.google.common.hash.Hashing
import org.slf4j.Logger
import xyz.ariane.util.dclog.lzDebug
import xyz.ariane.util.lang.RandomUtils
import xyz.ariane.util.lang.profile

/**
 * 负责探测某[EntityWrapper]是否有变化,使用[EntityWrapper.hashCode]和指定的[HashFunction]做判断
 *
 */
class ModificationTracker private constructor(
    val hotspotStatsActor: ActorRef, // 性能监控用的Actor
    val entityWrapper: EntityWrapper<*>, // 被跟踪对象
    private val logger: Logger,
    now: Long, // 当前时间
    immediately: Boolean, // 是否立即检测脏
    private val funnel: Funnel<EntityWrapper<*>>, // 用于序列化对象
    private val funnelSub: Funnel<Any> // 用于序列化对象
) {

    /** [entityWrapper]的key */
    val key: EntityWrapperKey = EntityWrapperKey(entityWrapper)

    /** 下次检查时间 */
    // SS：单位是ms
    internal var nextCheckTime: Long = 0L

    // 针对单个或多个字段的脏记录
    private val records: Array<ModificationTrackerRecord>

    /** 已经提交正在等待完成的操作 */
    internal var pendingOp: Op? = null

    /** 下一个需要执行的操作 */
    internal var nextOp: Op? = null

    /** 是否立即保存标记 */
    internal var needSaveImmediately: Boolean = false

    init {
        records = Array(entityWrapper.multiDataNum()) { ModificationTrackerRecord() }

        cleanup()

        // 更新下次检测时间
        if (!immediately) {
            updateNextCheckTime(now)
        } else {
            nextCheckTime = now // 立即检测脏
        }
    }

    /**
     * 标记所有记录为脏
     */
    fun markAllDataDirty() {
        records.forEach {
            it.dirty = true
        }
        if (entityWrapper.multiData()) {
            entityWrapper.updateMultiDirtyRecords(records)
        }
    }

    /**
     * 同步内部的hash值为最新值
     */
    fun cleanup() {
        // 计算粗检和细检的hashcode
        if (!entityWrapper.multiData()) {
            val record = records[0]
            record.lastIntHashCode = entityWrapper.dirtyHash()
            record.lastSerBytesHashCode = hash(entityWrapper)

        } else {
            val dataNum = entityWrapper.multiDataNum()
            val datas = entityWrapper.multiDatas()
            for (i in 0 until dataNum) {
                val record = records[i]
                val data = datas[i]

                record.lastIntHashCode = data.dirtyHash()
                record.lastSerBytesHashCode = hashSubData(data)
            }
        }

        logger.lzDebug { "Cleanup, key=$key" }
    }

    fun hasPendingOp(): Boolean = pendingOp != null

    fun hasNextOp(): Boolean = nextOp != null

    /**
     * 根据当前时间更新下次检测的时间
     */
    fun updateNextCheckTime(now: Long) {
        // 缩短随机时间，使集中在一起到时的entity分散开
        nextCheckTime = now + entityWrapper.fetchCheckModInterval().toMillis() - RandomUtils.between(0, 3000L)
    }

    fun isKnownDirty(): Boolean = pendingOp != null || nextOp != null

    fun whatWhyDirty(): String? {
        if (pendingOp != null) {
            return "DirtyPend - ${entityWrapper::class.java.simpleName}"
        } else if (nextOp != null) {
            return "DirtyNext - ${entityWrapper::class.java.simpleName}"
        } else {
            return null
        }
    }

    /**
     * 检查变化的级别
     */
    enum class CheckLevel {
        /** 优先使用[Object.hashCode]检查 */
        MINOR,

        /** 只使用[hash]散列对象序列化数据的方式检查 */
        FULL
    }

    /**
     * 检查跟踪的[EntityWrapper]是否有变化。
     *
     * 优先使用[EntityWrapper.hashCode]对比，如果对比[TRIGGER_FULL_CHECK_THRESHOLD]次都是相等的再序列化对象并使用[fullCheckHashFunction]计算hash code进行对比
     *
     * **注意：此方法返回true时内部的hash code值也会同步为最新值，即不进一步修改数据再次调用此方法将返回false**
     *
     * @return 如果有变化返回true， 否则返回false，如果[EntityWrapper.标记此对象的数据不会被修改_如果返回true变化将不会存库_危险慎用_危险慎用]为true
     */
    @JvmOverloads
    fun checkDirty(level: CheckLevel = CheckLevel.MINOR): Boolean {
        val ew = entityWrapper
        if (ew.标记此对象的数据不会被修改_如果返回true变化将不会存库_危险慎用_危险慎用) {
            return false
        }

        // 重置脏记录
        records.forEach { it.dirty = false }

        // 检测脏
        return if (!ew.multiData()) {
            val checkRt = try {
                check4Single(ew, level)
            } catch (e: Exception) {
                // 脏检查出现异常，直接认为脏了！走到这里，可能问题比较严重！
                logger.error("单数据脏检查出现异常（应该不是细检）: ew=${ew}", e)
                true
            }
            return checkRt
        } else {
            val dirty = check4Multiple(ew, level)
            if (dirty) {
                ew.updateMultiDirtyRecords(records)
            }
            dirty
        }
    }

    /**
     * 针对单数据的检测
     */
    private fun check4Single(ew: EntityWrapper<*>, level: CheckLevel): Boolean {
        val monitoring = turnOnPerfMonitoring

        // 粗检计算
        val newIntHashCode = profile(monitoring, hotspotStatsActor, "mhash_${ew.javaClass.name}") {
            // SS：计算对象本身的hash
            val newHash: Int
            try {
                newHash = ew.dirtyHash() // 粗检
            } catch (e: Exception) {
                logger.error("hashcode error: ew=${ew}", e)
                return true // hashCode异常时强制标记为已修改，防止数据丢失
            }

            return@profile newHash
        }

        val record = records[0]
        if (newIntHashCode != record.lastIntHashCode) {
            // 粗检值和之前的不同，认为脏了。
            record.lastIntHashCode = newIntHashCode

            try {
                record.lastSerBytesHashCode = hash(ew)
            } catch (e: Exception) {
                logger.error("计算细检hash出现异常 error: ew=${ew}", e)
            }

            record.intHashCodeEqualTimes = 0

            logger.lzDebug { "Modification found by int hash" }

            return true
        }

        if (level == CheckLevel.MINOR && record.intHashCodeEqualTimes <= TRIGGER_FULL_CHECK_THRESHOLD) {
            // SS：小级别，并且检测次数还没超过阈值
            ++record.intHashCodeEqualTimes
            return false

        } else {
            // 不是小级别，或者达到阈值了。
            record.intHashCodeEqualTimes = 0

            val newHashCode = profile(monitoring, hotspotStatsActor, "fhash_${ew.javaClass.name}") {
                // SS：再次计算完整hash
                val newHash: HashCode
                try {
                    newHash = hash(ew) // 细检
                } catch (e: Exception) {
                    logger.error("Full hash error: ew=${ew}", e)
                    return true // 强制保存防止丢失数据
                }

                return@profile newHash
            }

            val lsbh = record.lastSerBytesHashCode
            return if (lsbh == newHashCode) {
                false
            } else {
                record.lastSerBytesHashCode = newHashCode
                true
            }
        }
    }

    private fun check4Multiple(ew: EntityWrapper<*>, level: CheckLevel): Boolean {
        val monitoring = turnOnPerfMonitoring

        var hasDirty = false
        when (level) {
            CheckLevel.FULL -> {
                // 细检
                val dataNum = ew.multiDataNum()
                val datas = ew.multiDatas()
                for (i in 0 until dataNum) {
                    val record = records[i]
                    val data = datas[i]

                    // 直接进行细检
                    val newHashCode = profile(monitoring, hotspotStatsActor, "fhash_${data.javaClass.name}") {
                        // SS：再次计算完整hash
                        val newHash: HashCode
                        try {
                            newHash = hashSubData(data) // 细检
                        } catch (e: Exception) {
                            logger.error("Full hash error: ew=${ew}", e)
                            return true // 强制保存防止丢失数据
                        }

                        return@profile newHash
                    }

                    val lsbh = record.lastSerBytesHashCode
                    if (lsbh == newHashCode) {
                        // 没变化，跳过
                        continue
                    }

                    // 更新bin hash，重置粗检计数器
                    record.lastSerBytesHashCode = newHashCode
                    record.intHashCodeEqualTimes = 0

                    // 标记整个ew脏了
                    hasDirty = true
                    record.dirty = true

                    // 顺带进行粗检计算
                    val newIntHashCode = profile(monitoring, hotspotStatsActor, "mhash_${data.javaClass.name}") {
                        // SS：计算对象本身的hash
                        val newHash: Int
                        try {
                            newHash = data.dirtyHash() // 粗检
                        } catch (e: Exception) {
                            logger.error("hashcode error: ew=${ew}, data=${data}", e)
                            hasDirty = true
                            return@profile record.lastIntHashCode + 1 // hashCode异常时强制标记为已修改，防止数据丢失
                        }

                        return@profile newHash
                    }

                    if (newIntHashCode != record.lastIntHashCode) {
                        // 粗检值和之前的不同，更新粗检值
                        record.lastIntHashCode = newIntHashCode
                    }
                }
            }

            CheckLevel.MINOR -> {
                // 普通的例行粗检
                val dataNum = ew.multiDataNum()
                val datas = ew.multiDatas()
                for (i in 0 until dataNum) {
                    val record = records[i]
                    val data = datas[i]

                    // 粗检计算
                    val newIntHashCode = profile(monitoring, hotspotStatsActor, "mhash_${data.javaClass.name}") {
                        // SS：计算对象本身的hash
                        val newHash: Int
                        try {
                            newHash = data.dirtyHash() // 粗检
                        } catch (e: Exception) {
                            logger.error("hashcode error: ew=${ew}, data=${data}", e)
                            hasDirty = true
                            return@profile record.lastIntHashCode + 1 // hashCode异常时强制标记为已修改，防止数据丢失
                        }

                        return@profile newHash
                    }
                    if (newIntHashCode != record.lastIntHashCode) {
                        // 粗检值和之前的不同，认为脏了。
                        record.lastIntHashCode = newIntHashCode // 更新粗检值

                        record.intHashCodeEqualTimes = 0 // 重置计数器

                        record.lastSerBytesHashCode = hashSubData(data) // 计算并更新细检值
                        record.dirty = true // 标记这条记录脏了

                        hasDirty = true
                        continue // 脏了，完成这次检测
                    }

                    // 粗检通过，判断是否达到阈值，
                    if (record.intHashCodeEqualTimes <= TRIGGER_FULL_CHECK_THRESHOLD) {
                        // SS：小级别，并且检测次数还没超过阈值
                        ++record.intHashCodeEqualTimes
                        continue // 没达到阈值，完成这次检测
                    }

                    // - 达到阈值才会继续走下去

                    // 重置计数
                    record.intHashCodeEqualTimes = 0

                    // 细检
                    val newHashCode = profile(monitoring, hotspotStatsActor, "fhash_${data.javaClass.name}") {
                        // SS：再次计算完整hash
                        val newHash: HashCode
                        try {
                            newHash = hashSubData(data) // 细检
                        } catch (e: Exception) {
                            logger.error("Full hash error: ew=${ew}, data=${data}", e)
                            return true // 强制保存防止丢失数据
                        }

                        return@profile newHash
                    }

                    val lsbh = record.lastSerBytesHashCode
                    if (lsbh != newHashCode) {
                        record.lastSerBytesHashCode = newHashCode
                        record.dirty = true

                        hasDirty = true
                    }
                }
            }
        }

        return hasDirty
    }

    /**
     * 计算细检值
     */
    private fun hash(ew: EntityWrapper<*>): HashCode {
        return fullCheckHashFunction.hashObject(ew, funnel)
    }

    /**
     * 针对子数据计算细检值
     */
    private fun hashSubData(subData: Any): HashCode {
        return fullCheckHashFunction.hashObject(subData, funnelSub)
    }

    enum class Op {
        /** 对应数据库insert */
        SAVE,

        /** 对应数据库delete */
        DELETE,

        /** 对应数据库update */
        UPDATE,

        /** 对应数据库save on duplicate update */
        SAVE_OR_UPDATE,

        /** 表示放弃save，直接清除 */
        ABORT_SAVE,

        /** 表示需要替换对象实例 */
        REPLACE,
    }

    companion object {

        /** 是否打开性能统计，可在运行时动态修改，需要预先配置好日志输出 */
        @Volatile
        var turnOnPerfMonitoring = false

        /** 完整序列化比较时的hash函数 */
        private val fullCheckHashFunction: HashFunction = Hashing.goodFastHash(128)

        /** 在检查多少次int hash code相等时对比一次长hash code */
        const val TRIGGER_FULL_CHECK_THRESHOLD = 40

        @JvmStatic
        fun createKryoTracker(
            hotspotStatsActor: ActorRef, // 性能监控用的Actor
            ew: EntityWrapper<*>,
            logger: Logger,
            now: Long,
            immediately: Boolean,
            funnel: Funnel<EntityWrapper<*>>,
            funnelSub: Funnel<Any>
        ): ModificationTracker {
            return ModificationTracker(
                hotspotStatsActor,
                ew,
                logger,
                now,
                immediately,
                funnel,
                funnelSub
            )
        }

    }
}
