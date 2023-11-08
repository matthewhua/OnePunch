package xyz.ariane.util.akka

import akka.actor.*
import akka.event.Logging
import akka.event.LoggingAdapter
import akka.pattern.Patterns
import akka.util.Timeout
import scala.concurrent.Await
import scala.concurrent.duration.FiniteDuration
import xyz.ariane.util.lang.NamedRunnable
import xyz.ariane.util.lang.TickTask
import xyz.ariane.util.lang.castFuncOf
import xyz.ariane.util.dclog.lzDebug
import java.time.Instant
import java.util.concurrent.Executor
import java.util.concurrent.TimeUnit
import java.util.function.BiConsumer
import java.util.function.Supplier

//@Deprecated("Use coordinated shutdown instead.")
//fun gracefulShutdownShardRegion(actorSystem: ActorSystem, shardName: String) {
//    val logger = LoggerFactory.getLogger("gracefulShutdownShardRegion")
//
//    val shardRegion = ClusterSharding.get(actorSystem).shardRegion(shardName)
//    val inbox = Inbox.create(actorSystem)
//    inbox.watch(shardRegion)
//    inbox.send(shardRegion, ShardRegion.gracefulShutdownInstance())
//    val terminated = inbox.receive(FiniteDuration(30L, TimeUnit.DAYS)) //死等
//    when (terminated) {
//        is Terminated -> logger.info("ShardRegion(${terminated.actor}) actor terminated.")
//        else -> logger.error("Unexpected received msg {}", terminated)
//    }
//}

///**
// * 无限等待[shardName]指定的[ShardRegion]停止
// */
//@Deprecated("Use coordinated shutdown instead.")
//fun waitUntilShardRegionTerminate(actorSystem: ActorSystem, shardName: String) {
//    val logger = LoggerFactory.getLogger("gracefulShutdownShardRegion")
//    val shardRegion = ClusterSharding.get(actorSystem).shardRegion(shardName)
//    val inbox = Inbox.create(actorSystem)
//    inbox.watch(shardRegion)
//    val terminated = inbox.receive(FiniteDuration(30L, TimeUnit.DAYS)) //死等
//    when (terminated) {
//        is Terminated -> logger.info("ShardRegion(${terminated.actor}) actor terminated.")
//        else -> logger.error("Unexpected received msg {}", terminated)
//    }
//}

class AskException(msg: String) : RuntimeException(msg)

@JvmOverloads
fun <R> syncAsk(
        actor: ActorRef,
        msg: Any,
        expectedResClass: Class<R>,
        timeout: Timeout = Timeout(5, TimeUnit.SECONDS)
): R {
    val future = Patterns.ask(actor, msg, timeout)
    val resp: Any = Await.result(future, timeout.duration()) ?: throw AskException("Receive null response for $msg")
    return castFuncOf(expectedResClass).apply(resp)
}

@JvmOverloads
fun syncAsk(actor: ActorRef, msg: Any, timeout: Timeout = Timeout(5, TimeUnit.SECONDS)): Any? {
    val future = Patterns.ask(actor, msg, timeout)
    return Await.result(future, timeout.duration())
}

fun AbstractActor.scheduleTick(interval: Long, intervalUnit: TimeUnit, tickMsg: Any): Cancellable {
    val duration = FiniteDuration.apply(interval, intervalUnit)
    return context.system().scheduler()
            .schedule(duration, duration, self, tickMsg, context.dispatcher(), ActorRef.noSender())
}

/** 初始延迟的周期tick */
@Suppress("unused")
fun AbstractActor.scheduleDelayTick(delay: Long, interval: Long, unit: TimeUnit, tickMsg: Any): Cancellable {
    val delayDuration = FiniteDuration.apply(delay, unit)
    val intervalDuration = FiniteDuration.apply(interval, unit)
    return context.system().scheduler()
            .schedule(delayDuration, intervalDuration, self, tickMsg, context.dispatcher(), ActorRef.noSender())
}

fun AbstractActor.exec(taskName: String, func: () -> Unit) {
    require(taskName.isNotBlank()) { "taskName is blank" }

    self.tell(NamedRunnable(taskName, func), ActorRef.noSender())
}

fun AbstractActor.exec(task: NamedRunnable) {
    require(task.name.isNotBlank()) { "taskName is blank" }

    self.tell(task, ActorRef.noSender())
}

fun AbstractActor.execTickTask(taskName: String, func: () -> Unit) {
    require(taskName.isNotBlank()) { "taskName is blank" }

    self.tell(TickTask(taskName, func), ActorRef.noSender())
}

/** 将发给 */
fun ActorRef.asExecutor(): Executor = Executor { runnable -> this.tell(runnable, ActorRef.noSender()) }

/**
 * 简单的supervisor,可指定[SupervisorStrategy],生命周期与子actor相同,收到的消息都转发给子actor
 *
 */
class ProxySupervisor(private val childProps: Props, private val supervisorStrategy: SupervisorStrategy) :
        UntypedAbstractActor() {

    companion object {
        fun props(childProps: Props, supervisorStrategy: SupervisorStrategy): Props =
                Props.create(ProxySupervisor::class.java) { ProxySupervisor(childProps, supervisorStrategy) }
    }

    private lateinit var child: ActorRef

    override fun preStart() {
        child = context.actorOf(childProps)
        context.watch(child)
    }

    override fun supervisorStrategy(): SupervisorStrategy {
        return supervisorStrategy
    }

    override fun onReceive(msg: Any) {
        when (msg) {
            is Terminated -> if (child == msg.actor) context.stop(self) else child.forward(msg, context)
            else -> child.forward(msg, context)
        }
    }

}

/**
 * 通用的worker actor,可用于辅助其他actor异步执行任务,例如复杂计算,IO操作等.
 *
 * 使用[Worker]的actor需要接受[Runnable]作为消息并执行
 *
 * 支持[Runnable],可作为一般的Executor actor
 *
 * 注意,此actor只可本地使用不可远程部署
 *
 */
class Worker(val name: String) : UntypedAbstractActor() {

    val logger: LoggingAdapter = Logging.getLogger(context.system(), javaClass)

    class Job<R>(private val job: Supplier<out R>, private val callback: (BiConsumer<in R?, Throwable?>)? = null) :
            Runnable {
        /** 执行结果 */
        private var result: R? = null
        /** 异常 */
        private var exception: Throwable? = null

        fun doJob(worker: Worker) {
            try {
                result = job.get()
            } catch (e: Throwable) {
                exception = e
                worker.logger.error(e, "Job error")
            } finally {
                worker.sender.tell(this, ActorRef.noSender())
            }
        }

        /**
         * 用于交付给请求actor执行callback
         */
        override fun run() {
            callback?.accept(result, exception)
        }
    }

    companion object {
        fun props(name: String, dispatcher: String, mailbox: String): Props {
            return Props.create(Worker::class.java) { Worker(name) }.withDispatcher(dispatcher).withMailbox(mailbox)
        }
    }

    override fun preStart() {
        logger.lzDebug { "Worker [$name] started, self=$self" }
    }

    override fun postStop() {
        logger.lzDebug { "Worker [$name] stopped, self=$self" }
    }

    override fun onReceive(msg: Any) {
        when (msg) {
            is Job<*> -> msg.doJob(this)
            is Runnable -> msg.run()
            else -> unhandled(msg)
        }
    }
}

/**
 * 用于实现状态机类actor
 */
abstract class ActorState(val name: String) {
    open fun handleMsg(msg: Any) = Unit
    open fun tick(now: Instant) = Unit
    override fun toString() = name
}

