package xyz.ariane.util.concurrent

import akka.actor.ActorRef
import akka.pattern.Patterns
import akka.util.Timeout
import xyz.ariane.util.akka.asExecutor
import xyz.ariane.util.lang.castFuncOf
import java.time.Duration
import java.time.Instant
import java.time.temporal.ChronoUnit
import java.util.*
import java.util.concurrent.*
import java.util.function.*
import java.util.function.Function
import kotlin.reflect.KClass

val defaultAskTimeout = Timeout(10, TimeUnit.MINUTES)
val defaultAskTimeoutDuration = Duration.of(10, ChronoUnit.MINUTES)

val fastAskTimeout = Timeout(3, TimeUnit.SECONDS)

/**
 * 基于actor作为[Executor]实现的类[CompletionStage],用于按顺序实现混合异步操作、IO操作和询问其他actor等操作的复杂业务逻辑。
 *
 * 作为[Executor]的actor需要接受[Runnable]消息并执行
 *
 * 使用方法参考ActorCompletionStageTest
 *
 * **注意:操作在连续快速调用时是否和自己有race condition**
 *
 * @see [CompletionStage]
 * @see [CompletableFuture]
 */
class ActorCompletionStage<T> internal constructor(
    /** 用于超时调度的Executor */
    private val delayer: ScheduledThreadPoolExecutor,
    /** 主业务逻辑处理[Executor] */
    private val mainExecutor: Executor,
    /** 默认用于异步执行blocking io操作的[Executor] */
    private val defaultIoExecutor: Executor,
    /** 默认用于执行复杂计算(cpu密集型)的[Executor] */
    private val defaultComputationExecutor: Executor,
    /** 其他可按名字访问的[Executor] key:name */
    private val namedExecutors: Map<String, Executor>,
    /** @see [CompletableFuture] */
    private var cs: CompletionStage<T>? = null
) {

    private constructor(p: ActorCompletionStage<*>, cs: CompletionStage<T>)
            : this(p.delayer, p.mainExecutor, p.defaultIoExecutor, p.defaultComputationExecutor, p.namedExecutors, cs) {
        // SS：新的请求会创建新的ActorCompletionStage，也就是这里的this，然后老的请求对应的就是p。
        // SS：这里实际是让老的请求指向新的请求，也就是p指向this。
        p.childStage = this
    }

    /** 次stage的后续子stage，用于跟踪所有stage的完成情况 */
    // SS：这里的实现其实挺绕的。
    // SS：目前只需要知道新的请求会创建ActorCompletionStage，然后构造函数里面会让老的ActorCompletionStage的childStage指向新的。
    // SS：同时，新的ActorCompletionStage里面会保存当前请求的CompletionStage实例。
    private var childStage: ActorCompletionStage<*>? = null

    /** 是不是所有的子stage都完成了 */
    // SS：这里实际上就是从最老的请求判断到最新的请求。
    fun isAllChildDone(): Boolean {
        var stage: ActorCompletionStage<*>? = this
        while (stage != null) {
            if (!stage.isDone()) {
                return false
            }
            stage = stage.childStage
        }
        return true
    }

    /** 检查此stage是否已经完成 */
    private fun isDone(): Boolean = cs?.toCompletableFuture()?.let { it.isDone || it.isCancelled } ?: true

    private val initializedStage: CompletionStage<T>
        get() = requireNotNull(cs) { "Not initialized." }

    companion object {
        fun <T> create(
            delayer: ScheduledThreadPoolExecutor,
            mainActor: ActorRef,
            ioWorker: ActorRef,
            computationWorker: ActorRef,
            namedWorkers: Map<String, ActorRef> = emptyMap()
        ): ActorCompletionStage<T> {
            return ActorCompletionStage(
                delayer,
                mainActor.asExecutor(),
                ioWorker.asExecutor(),
                computationWorker.asExecutor(),
                namedWorkers.toExecutorMap()
            )
        }
    }

    /**
     * 创建一个已经完成的stage，直接指定结果值
     *
     * @param value 结果值
     *
     * @return this stage
     */
    fun completed(value: T): ActorCompletionStage<T> {
        cs = CompletableFuture.completedFuture(value)
        return this
    }

    /**
     * 通过IO操作获取的数据初始化，IO操作由io worker执行
     * @param supplier 提供[T]的IO操作
     *
     * @return this stage
     */
    fun supplyIo(supplier: Supplier<T>): ActorCompletionStage<T> {
        cs = CompletableFuture.supplyAsync(supplier, defaultIoExecutor)
        return this
    }

    fun supplyIo(executorName: String, supplier: Supplier<T>): ActorCompletionStage<T> {
        cs = CompletableFuture.supplyAsync(supplier, executorOf(executorName))
        return this
    }

    fun supplyIoTimeout(
        executorName: String,
        timeout: Long,
        unit: TimeUnit,
        supplier: Supplier<T>
    ): ActorCompletionStage<T> {
        cs = CompletableFuture
            .supplyAsync(supplier, executorOf(executorName))
            .applyToEither(timeoutAfter(timeout, unit)) { rt ->
                rt
            }
        return this
    }

    private fun <T> timeoutAfter(timeout: Long, unit: TimeUnit): CompletableFuture<T> {
        val result = CompletableFuture<T>()
        delayer.schedule({ result.completeExceptionally(TimeoutException()) }, timeout, unit)
        return result
    }

    private fun executorOf(executorName: String): Executor =
        requireNotNull(namedExecutors[executorName]) { "Executor $executorName not found." }

    /**
     * [supplyIo]'s kotlin API
     */
    inline fun supplyIoKt(crossinline resolve: () -> T): ActorCompletionStage<T> =
        supplyIo(Supplier { resolve() })

    /**
     * [supplyIo]'s kotlin API
     */
    inline fun supplyIoKt(executorName: String, crossinline resolve: () -> T): ActorCompletionStage<T> =
        supplyIo(executorName, Supplier { resolve() })

    /**
     * [supplyIoTimeout]'s kotlin API
     */
    inline fun supplyIoTimeoutKt(
        executorName: String,
        timeout: Long,
        unit: TimeUnit,
        crossinline resolve: () -> T
    ): ActorCompletionStage<T> =
        supplyIoTimeout(executorName, timeout, unit, Supplier { resolve() })

    /**
     * 通过请求某actor以返回的消息初始化
     *
     * @param actor 目标actor
     * @param message 请求消息
     * @param timeout 请求超时
     * @param checkResp 负责检查回复消息是否合法，由主actor执行检查
     *
     * @return this stage
     */
    fun askBase(
        actor: ActorRef,
        message: Any,
        timeout: Duration,
        checkResp: Function<in Any, out T>
    ): ActorCompletionStage<T> {
        cs = Patterns.ask(actor, message, timeout).thenApplyAsync(checkResp, mainExecutor)
//        cs = PatternsCS.ask(actor, message, timeout).thenApplyAsync(checkResp, mainExecutor)
        return this
    }

    /**
     * [askBase]的Kotlin API
     */
    inline fun askKt(
        actor: ActorRef,
        message: Any,
        timeout: Duration,
        crossinline checkResp: (Any) -> T
    ): ActorCompletionStage<T> =
        askBase(actor, message, timeout, Function { checkResp(it) })

    /**
     * 简化版[askBase]
     */
    fun ask(actor: ActorRef, message: Any, expectedResponseClass: Class<T>): ActorCompletionStage<T> =
        askBase(actor, message, defaultAskTimeoutDuration, castFuncOf(expectedResponseClass))

    /**
     * 简化版[askBase]
     */
    fun ask(actor: ActorRef, message: Any, timeout: Duration, expectedResponseClass: Class<T>): ActorCompletionStage<T> =
        askBase(actor, message, timeout, castFuncOf(expectedResponseClass))

    /**
     * 与此stage同时请求[actor]返回的结果[U]后，在主[mainExecutor]中执行[accumulate]合并此stage和[actor]返回结果，输出[V]
     *
     * @param actor 目标actor
     * @param message 请求消息
     * @param timeout 请求超时
     * @param checkResp 负责检查回复消息是否合法，由主actor执行检查
     * @param accumulate 负责合并当前stage和请求返回结果，由主actor执行
     *
     * @return 合并后的stage，结果类型为[V]
     */
    fun <U, V> andAskBase(
        actor: ActorRef,
        message: Any,
        timeout: Duration,
        checkResp: Function<in Any, out U>,
        accumulate: BiFunction<in T, in U, out V>
    ): ActorCompletionStage<V> {
        val askingStage: CompletionStage<U> = Patterns
            .ask(actor, message, timeout)
            .thenApplyAsync(checkResp, mainExecutor)
        val newStage: CompletionStage<V> = initializedStage
            .thenCombineAsync(askingStage, accumulate, mainExecutor) // 合并两个stage
        return ActorCompletionStage(this, newStage)
    }

    /**
     * [andAskBase]的Kotlin API
     */
    inline fun <U, V> andAskKt(
        actor: ActorRef,
        message: Any,
        timeout: Duration,
        crossinline checkResp: (Any) -> U,
        crossinline accumulate: (T, U) -> V
    ): ActorCompletionStage<V> =
        andAskBase(actor, message, timeout, Function { checkResp(it) }, BiFunction { t: T, u: U -> accumulate(t, u) })

    /**
     * 简化版[andAskBase]
     */
    fun <U, V> andAsk(
        actor: ActorRef,
        message: Any,
        expectedResponseClass: Class<U>,
        accumulate: BiFunction<in T, in U, out V>
    ): ActorCompletionStage<V> =
        andAskBase(actor, message, defaultAskTimeoutDuration, castFuncOf(expectedResponseClass), accumulate)

    /**
     * [andAskBase]的Kotlin API
     */
    inline fun <U : Any, V> andAskKt(
        actor: ActorRef,
        message: Any,
        expectedResponseClass: KClass<U>,
        crossinline accumulate: (T, U) -> V
    ): ActorCompletionStage<V> =
        andAsk(actor, message, expectedResponseClass.java, BiFunction { t: T, u: U -> accumulate(t, u) })

    /**
     * SS：在之前的请求完成后，再次发起一次请求，并用传入的消息作为消息。
     */
    fun <U, V> thenAsk(
        actor: ActorRef,
        message: Any,
        timeout: Duration,
        checkResp: Function<in Any, out U>,
        accumulate: BiFunction<in T, in U, out V>
    ): ActorCompletionStage<V> {
        val composedStage: CompletionStage<U> = initializedStage
            .thenComposeAsync(
                Function<T, CompletionStage<U>> {
                    Patterns
                        .ask(actor, message, timeout) // 这里使用的是传入的消息
                        .thenApplyAsync(checkResp, mainExecutor)
                }, mainExecutor
            )
        val newStage: CompletionStage<V> = initializedStage
            .thenCombineAsync(composedStage, accumulate, mainExecutor)
        return ActorCompletionStage(this, newStage)
    }

    /**
     * [thenAsk]的Kotlin API
     */
    inline fun <U, V> thenAskKt(
        actor: ActorRef,
        message: Any,
        timeout: Duration,
        crossinline checkResp: (Any) -> U,
        crossinline accumulate: (T, U) -> V
    ): ActorCompletionStage<V> =
        thenAsk(actor, message, timeout, Function { checkResp(it) }, BiFunction { t: T, u: U -> accumulate(t, u) })

    /**
     * 简化版[thenAsk]
     */
    fun <U, V> thenAsk(
        actor: ActorRef,
        message: Any,
        expectedResponseClass: Class<U>,
        accumulate: BiFunction<in T, in U, out V>
    ): ActorCompletionStage<V> =
        thenAsk(actor, message, defaultAskTimeoutDuration, castFuncOf(expectedResponseClass), accumulate)

    /**
     * [thenAsk]的Kotlin API
     */
    inline fun <U : Any, V> thenAskKt(
        actor: ActorRef,
        message: Any,
        expectedResponseClass: KClass<U>,
        crossinline accumulate: (T, U) -> V
    ): ActorCompletionStage<V> =
        thenAsk(actor, message, expectedResponseClass.java, BiFunction { t: T, u: U -> accumulate(t, u) })

    /**
     * 在此stage完成后，使用此stage的结果作为消息请求[actor]，返回以请求返回结果为结果的新stage
     */
    fun <U> thenAskWithResult(
        actor: ActorRef,
        timeout: Duration,
        checkResp: Function<in Any, out U>
    ): ActorCompletionStage<U> {
        val composedStage: CompletionStage<U> = initializedStage
            .thenComposeAsync(
                Function<T, CompletionStage<U>> { msg: T ->
                    Patterns
                        .ask(actor, msg, timeout) // 这里使用的是上次请求的结果消息。
                        .thenApplyAsync(checkResp, mainExecutor)
                }, mainExecutor
            )
        return ActorCompletionStage(this, composedStage)
    }

    /**
     * [thenAskWithResult]的Kotlin API
     */
    inline fun <U> thenAskWithResultKt(
        actor: ActorRef,
        timeout: Duration,
        crossinline checkResp: (Any) -> U
    ): ActorCompletionStage<U> =
        thenAskWithResult(actor, timeout, Function { checkResp(it) })


    /**
     * 通过复杂计算出的结果初始化，计算由 computation worker 负责执行
     *
     * @return this stage
     */
    fun compute(supplier: Supplier<T>): ActorCompletionStage<T> {
        cs = CompletableFuture.supplyAsync(supplier, defaultComputationExecutor)
        return this
    }

    /**
     * [compute]'s Kotlin API
     */
    inline fun computeKt(crossinline resolve: () -> T): ActorCompletionStage<T> =
        compute(Supplier { resolve() })

    private fun thenRun0(action: Runnable, executor: Executor): ActorCompletionStage<Void> {
        val newStage: CompletionStage<Void> = initializedStage.thenRunAsync(action, executor)
        return ActorCompletionStage(this, newStage)
    }

    private fun thenAccept0(action: Consumer<in T>, executor: Executor): ActorCompletionStage<Void> {
        val newStage: CompletionStage<Void> = initializedStage.thenAcceptAsync(action, executor)
        return ActorCompletionStage(this, newStage)
    }

    private fun <R> thenApply0(fn: Function<T, R>, executor: Executor): ActorCompletionStage<R> {
        val newStage: CompletionStage<R> = initializedStage.thenApplyAsync(fn, executor)
        return ActorCompletionStage(this, newStage)
    }

    /**
     * 此stage完成后，在主actor中执行，当前stage的结果（或者抛出的异常）作为参数传入[action]
     *
     * @see CompletionStage.whenCompleteAsync
     * @return 已完成的stage，包含与当前相同的结果
     */
    fun whenComplete(action: BiConsumer<in T, in Throwable?>): ActorCompletionStage<T> {
        val newStage: CompletionStage<T> = initializedStage.whenCompleteAsync(action, mainExecutor)
        return ActorCompletionStage(this, newStage)
    }

    /**
     * [whenComplete]'s kotlin API
     */
    inline fun whenCompleteKt(crossinline handle: (T?, Throwable?) -> Unit): ActorCompletionStage<T> =
        whenComplete(BiConsumer { t, u -> handle(t, u) })

    /**
     * 此stage完成后，在主actor中执行
     *
     * @see CompletionStage.thenRunAsync
     */
    private fun thenRun(action: Runnable): ActorCompletionStage<Void> =
        thenRun0(action, mainExecutor)

    /**
     * 在此stage完成后，在主actor中执行[action]
     *
     * @see CompletionStage.thenAcceptAsync
     */
    fun thenAccept(action: Consumer<in T>): ActorCompletionStage<Void> =
        thenAccept0(action, mainExecutor)

    /**
     * [thenAccept]的Kotlin API
     */
    inline fun thenAcceptKt(crossinline consume: (T) -> Unit): ActorCompletionStage<Void> =
        thenAccept(Consumer { consume(it) })

    /**
     * 在此stage完成后,在主actor中执行[fn],以fn返回的结果初始化新的stage返回
     *
     * @see CompletionStage.thenApplyAsync
     */
    fun <R> thenApply(fn: Function<T, R>): ActorCompletionStage<R> =
        thenApply0(fn, mainExecutor)

    /**
     * [thenApply]的Kotlin API
     */
    inline fun <R> thenApplyKt(crossinline fn: (T) -> R): ActorCompletionStage<R> =
        thenApply(Function { fn(it) })

    /**
     * 同[thenRun]，交给[defaultIoExecutor]执行
     */
    fun thenRunIo(action: Runnable): ActorCompletionStage<Void> =
        thenRun0(action, defaultIoExecutor)

    /**
     * [thenRunIo]'s Kotlin API
     */
    inline fun thenRunIoKt(crossinline action: () -> Unit): ActorCompletionStage<Void> =
        thenRunIo(Runnable { action() })

    /**
     * 同[thenAccept]，交给[defaultIoExecutor]执行
     */
    fun thenAcceptIo(action: Consumer<in T>): ActorCompletionStage<Void> =
        thenAccept0(action, defaultIoExecutor)

    /**
     * [thenAcceptIo]'s Kotlin API
     */
    inline fun thenAcceptIoKt(crossinline consume: (T) -> Unit): ActorCompletionStage<Void> =
        thenAcceptIo(Consumer { consume(it) })

    /**
     * 同[thenApply]，交给[defaultIoExecutor]执行
     */
    fun <R> thenApplyIo(fn: Function<T, R>): ActorCompletionStage<R> =
        thenApply0(fn, defaultIoExecutor)

    /**
     * [thenApplyIo]'s Kotlin API
     */
    inline fun <R> thenApplyIoKt(crossinline fn: (T) -> R): ActorCompletionStage<R> =
        thenApplyIo(Function { fn(it) })

    /**
     * 同[thenApply]，交给[namedExecutors]中指定的[Executor]执行
     */
    fun <R> thenApplyIo(executorName: String, fn: Function<T, R>): ActorCompletionStage<R> =
        thenApply0(fn, executorOf(executorName))

    /** [thenApplyIo]'s Kotlin API */
    inline fun <R> thenApplyIoKt(executorName: String, crossinline fn: (T) -> R): ActorCompletionStage<R> =
        thenApplyIo(executorName, Function { fn(it) })

    /**
     * 同[thenRun]，交给[defaultComputationExecutor]执行
     */
    fun thenRunComputing(action: Runnable): ActorCompletionStage<Void> =
        thenRun0(action, defaultComputationExecutor)

    /**
     * [thenRunComputing]'s Kotlin API
     */
    inline fun thenRunComputingKt(crossinline action: () -> Unit): ActorCompletionStage<Void> =
        thenRunComputing(Runnable { action() })

    /**
     * 同[thenAccept]，交给[defaultComputationExecutor]执行
     */
    fun thenAcceptComputing(action: Consumer<in T>): ActorCompletionStage<Void> =
        thenAccept0(action, defaultComputationExecutor)

    /**
     * [thenAcceptComputing]'s Kotlin API
     */
    inline fun thenAcceptComputingKt(crossinline action: (T) -> Unit): ActorCompletionStage<Void> =
        thenAcceptComputing(Consumer { action(it) })

    /**
     * 同[thenApply]，交给[defaultComputationExecutor]执行
     */
    fun <R> thenApplyComputing(fn: Function<T, R>): ActorCompletionStage<R> =
        thenApply0(fn, defaultComputationExecutor)

    /**
     * [thenApplyComputing]'s kotlin API
     */
    inline fun <R> thenApplyComputingKt(crossinline fn: (T) -> R): ActorCompletionStage<R> =
        thenApplyComputing(Function { fn(it) })

    /** only for testing */
    internal fun join() {
        cs?.toCompletableFuture()?.join()
    }

}

/**
 * 负责创建和跟踪[ActorCompletionStage]
 */
class ActorCompletionStageFactory(
    private val delayer: ScheduledThreadPoolExecutor,
    mainActor: ActorRef,
    defaultIoWorker: ActorRef,
    defaultComputationWorker: ActorRef,
    namedWorkers: Map<String, ActorRef>,
    private val waitingPendingTimeoutSeconds: Long = 60L
) {
    private val mainExecutor = mainActor.asExecutor()
    private val defaultIoExecutor = defaultIoWorker.asExecutor()
    private val defaultComputationExecutor = defaultComputationWorker.asExecutor()
    private val namedExecutors = namedWorkers.toExecutorMap()

    /** 等待完成的根stage列表 */
    private val pendingRootCompletionStages: LinkedList<ActorCompletionStage<*>> = LinkedList()

    private var startWaitingPendingCSTime: Instant? = null

    fun hasPendingStages(): Boolean = !pendingRootCompletionStages.isEmpty()

    fun startWaitingPendings(now: Instant) {
        startWaitingPendingCSTime = now
    }

    /** 检查等待未完成的ACS链是否超时，同时会清理已完成的ACS */
    fun checkWaitingPendingTimeout(now: Instant): Boolean {
        val startTime = requireNotNull(startWaitingPendingCSTime) { "Call startWaitingPendingCSTime first." }
        cleanAllDoneStages()
        if (hasPendingStages()) {
            val waitingSeconds = startTime.until(now, ChronoUnit.SECONDS)
            if (waitingSeconds > waitingPendingTimeoutSeconds) {
                return true
            }
        }
        return false
    }

    fun countPendingStages(): Int = pendingRootCompletionStages.size

    fun <T> create(): ActorCompletionStage<T> {
        return ActorCompletionStage<T>(
            delayer,
            mainExecutor,
            defaultIoExecutor,
            defaultComputationExecutor,
            namedExecutors
        ).also {
            cleanAllDoneStages()
            pendingRootCompletionStages.add(it)
        }
    }

    private fun cleanAllDoneStages() {
        var count = 0
        val iterator = pendingRootCompletionStages.iterator()
        while (iterator.hasNext()) {
            val stage = iterator.next()
            if (stage.isAllChildDone()) {
                iterator.remove()
                ++count
            }
        }
    }

}

private fun Map<String, ActorRef>.toExecutorMap(): Map<String, Executor> =
    map { (name, actorRef) -> name to actorRef.asExecutor() }.toMap()