package xyz.ariane.util.memodb

import akka.actor.Actor
import akka.actor.ActorRef
import com.google.common.collect.ClassToInstanceMap
import com.google.common.collect.MutableClassToInstanceMap
import org.slf4j.Logger
import xyz.ariane.util.concurrent.ActorCompletionStage
import xyz.ariane.util.lang.TickTimer
import xyz.ariane.util.lang.joinL
import xyz.ariane.util.dclog.lzDebug
import xyz.ariane.util.memodbupgrade.AbstractVersionDbDataUpgrade
import xyz.ariane.util.monitor.WdbStatsReporter
import java.time.Duration
import java.time.Instant
import java.util.*

/**
 * 一个模块的内存数据容器
 *
 */
interface DataContainer {

    /**
     * 从数据库加载所有数据
     *
     * **注意：不在所有者Actor中执行，需要保证线程安全**
     *
     * @param ownerId 所有者id
     * @return 加载的数据
     */
    fun load(ownerId: Any, dao: CommonDao): Any?

    /**
     * 初始化，在所有者Actor中执行
     *
     * @param owner 所有者actor
     * @param data [load]返回的数据
     * @param depDCRepo 从这里获取其他依赖的[DataContainer]
     */
    fun init(owner: Actor, data: Any?, depDCRepo: DataContainerRepo)

}

/**
 * 可直接按[Class]获取的[DataContainer]库
 */
interface DataContainerRepo {
    fun <T : DataContainer> getDC(containerType: Class<T>): T
}

/**
 * [DataContainer]管理器，按需懒加载业务所需的[DataContainer]
 *
 * @param dbReadWorker 在使用ACS执行数据库查询操作时用的worker名字，数据加载操作应该用独立的worker在独立的dispatcher上执行
 * @param fetchDao 如何获取[CommonDao]
 * @param dependencyMap [DataContainer]的依赖关系，不支持多级依赖
 * @param jdbcBatchSize jdbc的批量操作大小，需要与下层jdbc连接的配置一致
 * @param wdbLogger [WriteOnlyInMemoryDb4Json]使用的logger
 * @param wdbTickCycle [WriteOnlyInMemoryDb4Json]的tick周期数
 * @param wdbMaxTickDuration 设置[WriteOnlyInMemoryDb4Json.maxTickDurationNanos]
 * @param entityAppenderFlushTickCycle [LogLikeEntityAppender]的tick周期数
 */
abstract class LazyDataContainerManager(
    private val wdbType: Int,
    private val kryoUtil: KryoUtil,
    private val dbReadWorker: String,
    private val fetchDao: () -> CommonDao,
    private val dependencyMap: Map<Class<out DataContainer>, List<Class<out DataContainer>>>,
    jdbcBatchSize: Int,
    wdbStatsReporter: WdbStatsReporter,
    perfMonitorActor: ActorRef,
    hotspotStatsActor: ActorRef,
    val wdbLogger: Logger,
    val esErrorLogger: (Throwable, String) -> Unit, // 专门用于记录错误日志的方法
    wdbTickCycle: Int = 2,
    wdbMaxTickDuration: Duration = Duration.ofMillis(1L),
    entityAppenderFlushTickCycle: Int = 10,
    upgradeHandlers: List<AbstractVersionDbDataUpgrade>, // 数据库数据升级列表
) {

    protected abstract val owner: Actor

    protected abstract val ownerId: Any

    @Suppress("LeakingThis")
    val wdb = WriteOnlyInMemoryDb4Json(
        wdbType,
        false,
        10,
        kryoUtil,
        createACS = this::createACS,
        fetchDao = fetchDao,
        logger = wdbLogger,
        wdbStatsReporter = wdbStatsReporter,
        esLogger = esErrorLogger,
        syncOpSubmissionListener = this::onSyncOpSubmitted,
        batchSize = jdbcBatchSize,
        maxTickDuration = wdbMaxTickDuration,
        perfMonitorActor = perfMonitorActor,
        hotspotStatsActor = hotspotStatsActor,
        upgradeHandlers = upgradeHandlers,
    )

    private val wdbTickTimer = TickTimer(wdbTickCycle)

    /** protected for test */
    // SS：加载完毕的数据容器表
    protected val dataContainerMap: ClassToInstanceMap<DataContainer> = MutableClassToInstanceMap.create()

    private val containerRepo: InnerContainerRepo = InnerContainerRepo()

    /** 正在加载数据并初始化的DC */
    private val loadingContainers: MutableSet<Class<out DataContainer>> = hashSetOf()

    /** 等待所需DC初始化完成才能执行的操作 */
    private val pendingRequirementQueue: Queue<PendingRequirement> = LinkedList()

    protected abstract fun <T> createACS(): ActorCompletionStage<T>

    /**
     * 直接获取一个[DataContainer]
     *
     * **注意：如果还未初始化抛出[IllegalStateException]**
     */
    @Suppress("UNCHECKED_CAST")
    protected fun <T : DataContainer> getDC(containerType: Class<T>): T {
        return dataContainerMap[containerType] as? T ?: error("Missing dc: $containerType")
    }

    @Suppress("UNCHECKED_CAST")
    fun <T : DataContainer> getDCIfPresent(containerType: Class<T>): T? {
        return dataContainerMap[containerType] as T?
    }

    inner class InnerContainerRepo : DataContainerRepo {
        override fun <T : DataContainer> getDC(containerType: Class<T>): T =
            this@LazyDataContainerManager.getDC(containerType)
    }

    // SS：等待处理的IO请求
    class PendingRequirement(val missingContainers: MutableList<Class<out DataContainer>>, val handle: () -> Unit)

    /**
     * 动态的require支持，在[proc]执行时保证[containerTypes]列表中的[DataContainer]都已经初始化完成
     *
     * **注意：慎用此方法，没有编译器静态检查保证，在[proc]中获取[containerTypes]不包含的[DataContainer]时可能抛出异常**
     */
//    fun unsafeRequire(containerTypes: Set<Class<out DataContainer>>, proc: Procedure<DataContainerRepo>) {
//        handleAll(containerTypes) { proc.apply(containerRepo) }
//    }

    /** Kotlin API */
    fun unsafeRequireKt(
        containerTypes: Set<Class<out DataContainer>>,
        failedHandle: DbLoadFailedHandle?,
        handle: (DataContainerRepo) -> Unit
    ) {
        handleAll(containerTypes, failedHandle) { handle(containerRepo) }
    }

    protected open fun handleAll(
        containerTypes: Collection<Class<out DataContainer>>,
        failedHandle: DbLoadFailedHandle?,
        handle: () -> Unit
    ) {
        val missingList = LinkedList<Class<out DataContainer>>()
        containerTypes.filterTo(missingList) { !dataContainerMap.containsKey(it) }
        if (missingList.isEmpty()) {
            // SS：直接处理
            handle()
        } else {
            // SS：遍历缺失表，添加缺失表中的元素所依赖的其他容器，从而能关联加载
            missingList
                .flatMap { clazz -> dependencyMap[clazz].orEmpty() }
                .forEach { depClazz ->
                    missingList.remove(depClazz) // 如果有先移除，不重复加载
                    missingList.addFirst(depClazz) // 被依赖的先初始化
                }

            // 加载异常时可能会有内存泄露，防止OutOfMemoryError
            check(pendingRequirementQueue.size <= 10000) {
                "Too many pending requirements, n=${pendingRequirementQueue.size}"
            }

            // SS：添加到请求队列中
            pendingRequirementQueue.offer(
                PendingRequirement(
                    missingList,
                    handle
                )
            )

            // SS：开始增量加载和初始化
            incrementalLoadAndInitialize(missingList, failedHandle)
        }
    }

    /**
     * SS：增量加载和初始化
     * SS：会被[handleAll]调用
     */
    protected open fun incrementalLoadAndInitialize(
        missingContainers: List<Class<out DataContainer>>,
        failedHandle: DbLoadFailedHandle?
    ) {
        val actualMissing = missingContainers - loadingContainers // SS：看还差哪些容器没加载

        wdbLogger.lzDebug { "Save ${missingContainers.size - actualMissing.size} loadings." }

        if (actualMissing.isEmpty()) {
            return
        } else {
            wdbLogger.lzDebug {
                """Incremental load
        |${actualMissing.map(Class<*>::getSimpleName).joinL()}
        |loading=${loadingContainers.size}, req=${pendingRequirementQueue.size}
        |""".trimMargin()
            }
        }

        // SS：添加到加载队列中
        loadingContainers += actualMissing

        // SS：创建ACS，开始异步加载
        createACS<List<Pair<DataContainer, Any?>>>()
            .supplyIoKt(executorName = dbReadWorker) {
                actualMissing.map { containerClass ->
                    val container = containerClass.newInstance()
                    val data = container.load(ownerId, fetchDao()) // SS：开始加载
                    Pair(container, data) // SS：返回容器数据对
                }
            }
            .whenCompleteKt { result, err ->
                try {
                    when {
                        err != null -> {
                            esErrorLogger(err, "Owner $ownerId, load data failed, containers=$missingContainers")
                            handleLoadingException(err)

                            // 下面是尝试执行一些补救错误
                            if (failedHandle != null) {
                                failedHandle.handleLoadingException()
                            }
                        }
                        result != null -> {
                            try {
                                // SS：处理加载结果
                                initAllContainers(result)

                            } catch (e: Throwable) {
                                esErrorLogger(e, "Owner $ownerId, init dc failed, data=$result")
                                handleInitializingException(e)

                                // 下面是尝试执行一些补救错误
                                if (failedHandle != null) {
                                    failedHandle.handleInitializingException()
                                }
                                return@whenCompleteKt
                            }
                        }
                        else -> error("Impossible")
                    }

                } catch (e: Throwable) {
                    esErrorLogger(e, "Owner $ownerId")
                }
            }
    }

    /**
     * SS：初始化所有数据容器
     * SS：这个方法会被[incrementalLoadAndInitialize]方法调用
     */
    protected open fun initAllContainers(result: List<Pair<DataContainer, Any?>>) {
        for ((container, data) in result) {
            if (dataContainerMap.containsKey(container.javaClass)) {
                // SS：已经初始化了
                wdbLogger.lzDebug { "${container.javaClass.simpleName} already initialized. Drop this one." }

            } else {
                // SS：初始化
                container.init(owner, data, containerRepo)

                // SS：添加到容器表中
                dataContainerMap[container.javaClass] = container

                wdbLogger.lzDebug { "${container.javaClass.simpleName} initialized." }
            }

            // SS：执行容器初始化完毕的回调
            onContainerInitialized(container.javaClass)
        }
    }

    protected open fun onContainerInitialized(containerClass: Class<DataContainer>) {
        // SS：从正在加载表中移除
        loadingContainers -= containerClass

        // SS：从请求的缺失容器表中移除
        for (pendingRequirement in pendingRequirementQueue) {
            pendingRequirement.missingContainers -= containerClass
        }

        // SS：清理掉完成的请求，并执行这个请求关联的handle
        var head = pendingRequirementQueue.peek()
        while (head != null) {
            if (head.missingContainers.isEmpty()) {
                // SS：清理
                pendingRequirementQueue.poll()

                // SS：执行handle
                exec("handle_requirement", head.handle)
            } else {
                break // 为了保证实际执行的顺序于代码书写顺序一致，即使后面有已经满足条件的请求，也要等待
            }
            head = pendingRequirementQueue.peek()
        }
    }

    /** 处理[DataContainer.load]的异常 */
    protected abstract fun handleLoadingException(e: Throwable)

    /** 处理[DataContainer.init]的异常 */
    protected abstract fun handleInitializingException(e: Throwable)

    /** 用于监听[WriteOnlyInMemoryDb4Json]的[SyncOp]执行操作 */
    protected open fun onSyncOpSubmitted(syncOp: SyncOp) = Unit

    protected abstract fun exec(name: String, task: () -> Unit)

    fun isAllClean(): Boolean = !wdb.hasPendingNextSyncOps()

    /**
     * 供心跳调用的方法
     */
    open fun tick(now: Instant) {
        wdbTickTimer.whenTimeUp {
            wdb.tick(now)
//            exec("tt_wdb_tick") { wdb.tick(now) }
        }
    }
}
