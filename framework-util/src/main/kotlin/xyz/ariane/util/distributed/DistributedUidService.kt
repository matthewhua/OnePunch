package xyz.ariane.util.distributed

import akka.actor.*
import akka.actor.SupervisorStrategy.resume
import akka.actor.SupervisorStrategy.stop
import akka.cluster.Cluster
import akka.cluster.client.ClusterClient
import akka.cluster.client.ClusterClientReceptionist
import akka.cluster.routing.ClusterRouterPool
import akka.cluster.routing.ClusterRouterPoolSettings
import akka.cluster.singleton.ClusterSingletonManager
import akka.cluster.singleton.ClusterSingletonManagerSettings
import akka.cluster.singleton.ClusterSingletonProxy
import akka.cluster.singleton.ClusterSingletonProxySettings
import akka.event.Logging
import akka.event.LoggingAdapter
import akka.japi.pf.ReceiveBuilder
import akka.routing.RoundRobinPool
import akka.serialization.JSerializer
import xyz.ariane.util.distributed.DUidMessage.*
import scala.concurrent.duration.Duration
import scala.concurrent.duration.FiniteDuration
import xyz.ariane.util.dclog.lzDebug
import xyz.ariane.util.akka.syncAsk
import java.nio.ByteBuffer
import java.time.Clock
import java.time.ZonedDateTime
import java.time.temporal.ChronoUnit
import java.util.concurrent.TimeUnit
import java.time.Duration as JDuration

const val DUID_SERVICE: String = "duidService"
const val DUID_SERVICE_PROXY: String = "${DUID_SERVICE}Proxy"
const val DUID_SINGLETON_MASTER: String = "/user/$DUID_SERVICE/singleton"
const val DUID_LOCAL_WORKER_ROUTER = "duidLocalWorkerRouter"

/** 集群外部系统请求生成uid时，用此函数创建请求消息 */
fun createCCUidRequest(): ClusterClient.Send = ClusterClient.Send(DUID_SINGLETON_MASTER, UidRequest)

/**
 * 在本节点启动duid service，所有需要生成uid的节点都需要执行，并且节点的role中必须包含[workerUseRole]指定的角色
 *
 * **注意：此方法会阻塞的检测本地duid服务的可用性，如果检测超时会抛出[IllegalStateException]**
 *
 * 本服务有两种使用方式:
 *  1. 将[UidRequest]发给[DUidMaster]，再转发给[DUidWorker]，[DUidWorker]直接回复请求者。此方式会跨间点发送消息。
 *  2. 将本地[Cluster]节点添加[workerUseRole]指定的角色，则可以请求本地的[DUidLocalWorkerRouter]，转发给本地的[DUidWorker]回复请求。
 *
 * 推荐优先使用方式2，因为性能更好，并且不存在[DUidMaster]的各种SPOF问题。
 * 但方式2依赖本地部署worker，如果无法本地部署，或者Uid生成服务部署在其他集群，可使用方式1。
 *
 * @param system 本地[ActorSystem]
 * @param masterUseRole master所在的节点角色
 * @param workerUseRole worker所在的节点角色
 * @param maxInstancePerNode 每个节点的最多的worker数
 * @param dispatcher master和worker共用的dispatcher
 * @param mailbox master和worker使用的mailbox
 * @param checkAvailableTimeout 服务可用性检查超时时间，超过此时间服务仍不可用，则抛出[IllegalStateException]，默认为5min
 *
 * @return 返回[DUidLocalWorkerRouter]的[ActorRef]
 */
@JvmOverloads
fun startDuidService(
    system: ActorSystem,
    masterUseRole: String,
    workerUseRole: String,
    maxInstancePerNode: Int,
    dispatcher: String,
    mailbox: String,
    checkAvailableTimeout: JDuration = JDuration.of(5, ChronoUnit.MINUTES)
): ActorRef {

    // 只有master所在的节点角色才需要启动ClusterSingletonManager
    if (masterUseRole in Cluster.get(system).selfRoles) {
        val singletonSettings = ClusterSingletonManagerSettings.create(system).withRole(masterUseRole)
        val props = DUidMaster.props(workerUseRole, maxInstancePerNode, dispatcher, mailbox)
        system.actorOf(ClusterSingletonManager.props(props, PoisonPill.getInstance(), singletonSettings), DUID_SERVICE)
    }

    val proxySettings = ClusterSingletonProxySettings.create(system).withRole(masterUseRole)
    system.actorOf(ClusterSingletonProxy.props("/user/$DUID_SERVICE", proxySettings), DUID_SERVICE_PROXY)

    val localRouter = system.actorOf(DUidLocalWorkerRouter.props(dispatcher, mailbox), DUID_LOCAL_WORKER_ROUTER)

    val logger: LoggingAdapter = Logging.getLogger(system, "start_duid_service")
    val testAvailableStartTime = ZonedDateTime.now()
    var available = false
    do {
        TimeUnit.SECONDS.sleep(1L)
        val now = ZonedDateTime.now()
        if (now.isAfter(testAvailableStartTime.plus(checkAvailableTimeout))) {
            throw IllegalStateException("Test local duid-service failed after retrying for $checkAvailableTimeout")
        }
        try {
            val response = syncAsk(localRouter, UidRequest)
            if (response is UidResponse) {
                available = true
                logger.info("Test local duid-service succeeded!")
            } else {
                logger.lzDebug { "Test local duid-service got $response" }
            }
        } catch (e: Exception) {
            logger.warning("Sync ask local router has exception: ${e.javaClass.name}, message=${e.message}")
        }
    } while (!available)

    return localRouter
}

/**
 * The starting epoch millis of timestamp field.
 * 2016-01-02T15:00+08:00[Asia/Shanghai]
 */
private const val EPOCH_BASE: Long = 1451718000000L

private const val SEQUENCE_BITS = 16
private const val WORKER_ID_BITS = 8
private const val TIMESTAMP_BITS = 39

internal const val SEQUENCE_SHIFT = TIMESTAMP_BITS
internal const val WORKER_ID_SHIFT = TIMESTAMP_BITS + SEQUENCE_BITS

internal const val MAX_SEQUENCE: Int = (1 shl SEQUENCE_BITS) - 1
internal const val MAX_WORKER_ID: Int = (1 shl WORKER_ID_BITS) - 1
internal const val MIN_WORKER_ID: Int = 1

const val MAX_TIMESTAMP: Long = (1L shl TIMESTAMP_BITS) - 1

private const val BYTE_BYTES: Int = java.lang.Byte.BYTES
private const val INT_BYTES: Int = Integer.BYTES
private const val LONG_BYTES: Int = java.lang.Long.BYTES

internal val resumeSupervisorStg = OneForOneStrategy(-1, Duration.Inf()) { ex: Throwable ->
    when (ex) {
        is ActorInitializationException -> stop()
        is ActorKilledException -> stop()
        is DeathPactException -> stop()
        else -> resume()
    }
}

/**
 * 负责分配worker id和分发id生成请求
 *
 * 每个worker每毫秒可以生成最多65536个唯一id，格式如下,
 *
 * sign workerId  --- sequence --- ---------- timestamp in millis --------
 * 64   63    56  55            40 39                                    1
 * 0    00000000  0000000000000000 000000000000000000000000000000000000000
 *
 * 1 bit sign = 0
 * 8 bits worker id, unsigned, range: [1,255] (0 not used)
 * 16 bits sequence id, unsigned, range: [0,65535]
 * 39 bits timestamp in millis, up to 2033-6-4 12:56:53
 *
 */
class DUidMaster(
    val workerUseRole: String?,
    val maxInstancePerNode: Int,
    val dispatcher: String,
    val workerMailBox: String,
    val useCluster: Boolean = true,
    /** Only for testing */
    val createWorkerProps: ((String, String) -> Props)? = null
) : AbstractActorWithStash() {

    companion object {
        fun props(
            workerUseRole: String? = null,
            maxInstancePerNode: Int = 1,
            dispatcher: String,
            mailbox: String
        ): Props =
            Props.create(DUidMaster::class.java) { DUidMaster(workerUseRole, maxInstancePerNode, dispatcher, mailbox) }
                .withMailbox(mailbox)
                .withDispatcher(dispatcher)
    }

    private val logger: LoggingAdapter = Logging.getLogger(context.system(), javaClass)

    private lateinit var workerRouter: ActorRef

    private var workerPoolRestarting: Boolean = false

    private var workerIdSeq: Int = MIN_WORKER_ID

    private fun createWorkerRouter(): ActorRef {
        val workerProps =
            createWorkerProps?.invoke(dispatcher, workerMailBox) ?: DUidWorker.props(dispatcher, workerMailBox)
        val routerProps = if (useCluster) {
            ClusterRouterPool(
                RoundRobinPool(0).withSupervisorStrategy(resumeSupervisorStg),
                ClusterRouterPoolSettings(
                    MAX_WORKER_ID,
                    maxInstancePerNode,
                    true,
                    setOf(workerUseRole)
                )
            ).props(workerProps)
        } else {
            RoundRobinPool(maxInstancePerNode).withSupervisorStrategy(resumeSupervisorStg).props(workerProps)
        }
        return context.actorOf(routerProps, "workerRouter")
    }

    override fun preStart() {
        workerRouter = createWorkerRouter()
        if (useCluster) {
            ClusterClientReceptionist.get(context.system()).registerService(self)
        }
        logger.info("$self started.")
    }

    override fun postStop() {
        if (useCluster) {
            ClusterClientReceptionist.get(context.system()).unregisterService(self)
        }
        Thread.sleep(2L) // 防止垮节点迁移过快导致id生成重复
        logger.info("$self stopped.")
    }

    override fun createReceive(): Receive = ReceiveBuilder.create().matchAny(this::onReceive).build()

    private fun onReceive(msg: Any) {
        when (msg) {
            is TakeWorkerId -> assignWorkerId()
            is UidRequest -> forwardReqToWorker(msg)
            is Terminated -> tryFinishWorkerPoolRestart(msg)
        }
    }

    private fun forwardReqToWorker(msg: Any) {
        if (workerPoolRestarting) {
            stash()
        } else {
            workerRouter.forward(msg, context)
        }
    }

    private fun tryFinishWorkerPoolRestart(msg: Terminated) {
        if (workerPoolRestarting && msg.actor == workerRouter) {
            workerRouter = createWorkerRouter()
            workerPoolRestarting = false
            unstashAll()
        }
    }

    private fun assignWorkerId() {
        if (workerIdSeq >= MAX_WORKER_ID) {
            logger.warning("Max worker id $MAX_WORKER_ID reached, restarting all workers...")
            workerPoolRestarting = true

            Thread.sleep(2L) // 防止worker重启过快(<1ms)导致id生成重复

            context.watch(workerRouter)
            context.stop(workerRouter)
            workerIdSeq = MIN_WORKER_ID
        } else if (!workerPoolRestarting) {
            sender.tell(WorkerId(workerIdSeq++), self)
        } else {
            logger.lzDebug { "Worker pool is restarting, drop assign worker id request." }
        }
    }

}

class DUidWorker(private val clock: Clock = Clock.systemDefaultZone()) : AbstractActorWithStash() {

    enum class State {
        INIT,
        UP,
    }

    companion object {
        fun props(dispatcher: String, mailbox: String): Props =
            Props.create(DUidWorker::class.java)
                .withMailbox(mailbox)
                .withDispatcher(dispatcher)
    }

    val logger: LoggingAdapter = Logging.getLogger(context.system(), javaClass)

    private var state: State = State.INIT

    var id: Int = -1

    private var lastTimestamp: Long = currentTimeMillis()

    private var sequence: Int = 0

    private var cancelTakeWorkerId: Cancellable? = null

    private var cancelRegisterToLocalRouter: Cancellable? = null

    override fun preStart() {
        scheduleTakeWorkerId()
        scheduleRegisterToLocalRouter()
        logger.info("$self started.")
    }

    private fun scheduleRegisterToLocalRouter() {
        val localRouter = selectLocalRouter()
        val interval = FiniteDuration(1, TimeUnit.SECONDS)
        cancelRegisterToLocalRouter = context.system().scheduler()
            .schedule(
                interval,
                interval,
                Runnable { localRouter.tell(DUidLocalWorkerRouter.Cmd.RegisterWorker, self) },
                context.dispatcher()
            )
    }

    private fun selectLocalRouter(): ActorSelection = context.actorSelection("/user/$DUID_LOCAL_WORKER_ROUTER")

    private fun scheduleTakeWorkerId() {
        val masterProxy = context.actorSelection("/user/$DUID_SERVICE_PROXY")
        val interval = FiniteDuration(1, TimeUnit.SECONDS)
        cancelTakeWorkerId = context.system().scheduler()
            .schedule(interval, interval, Runnable { masterProxy.tell(TakeWorkerId, self) }, context.dispatcher())
    }

    override fun postStop() {
        selectLocalRouter().tell(DUidLocalWorkerRouter.Cmd.UnRegisterWorker, self)
        cancelTakeWorkerId?.cancel()
        cancelRegisterToLocalRouter?.cancel()
        logger.info("Worker $id stopped, $self")
    }

    override fun createReceive(): Receive = ReceiveBuilder.create().matchAny(this::onReceive).build()

    private fun onReceive(msg: Any) {
        when (msg) {
            DUidLocalWorkerRouter.Cmd.RegisterOk -> cancelRegisterToLocalRouter?.cancel()
            else -> {
                when (state) {
                    State.UP -> if (msg === UidRequest) {
                        sender.tell(UidResponse(nextUid()), ActorRef.noSender())
                    }
                    State.INIT -> when (msg) {
                        is WorkerId -> {
                            cancelTakeWorkerId?.cancel()
                            id = msg.id
                            state = State.UP
                            unstashAll()
                            logger.info("Worker $id is up, $self")
                        }
                        else -> stash()
                    }
                }
            }
        }
    }

    private fun currentTimeMillis(): Long = clock.millis()

    private fun nextUid(): Long {
        val last = lastTimestamp
        var now = currentTimeMillis()
        if (now < last) {
            sender.tell(Reject, ActorRef.noSender())
            throw InvalidSystemClock("Clock is moved backwards. Refusing requests for ${last - now}ms, lastTimestamp: $last")
        } else if (now == last) {
            sequence = (sequence + 1) and MAX_SEQUENCE
            if (sequence == 0) {
                now = loopUntilNextMillis(last)
            }
        } else {
            sequence = 0
        }
        lastTimestamp = now

        return (id.toLong() shl WORKER_ID_SHIFT) or
                (sequence.toLong() shl SEQUENCE_SHIFT) or
                (now - EPOCH_BASE)
    }

    private fun loopUntilNextMillis(last: Long): Long {
        var now = currentTimeMillis()
        while (now == last) {
            now = currentTimeMillis()
        }
        return now
    }
}

class InvalidSystemClock(message: String) : RuntimeException(message)

// messages

sealed class DUidMessage {
    object TakeWorkerId : DUidMessage()

    class WorkerId(val id: Int) : DUidMessage()

    object UidRequest : DUidMessage()

    class UidResponse(val uid: Long) : DUidMessage()

    object Reject : DUidMessage()
}

class DUidMessageSerializer : JSerializer() {

    private val takeWorkerIdType: Byte = 1
    private val workerIdType: Byte = 2
    private val uidRequestType: Byte = 3
    private val uidResponseType: Byte = 4
    private val rejectType: Byte = 5

    override fun identifier(): Int {
        return 77573139
    }

    override fun toBinary(o: Any): ByteArray = when (o) {
        UidRequest -> ByteBuffer.allocate(BYTE_BYTES).put(uidRequestType).array()
        is UidResponse -> ByteBuffer.allocate(BYTE_BYTES + LONG_BYTES).put(uidResponseType).putLong(o.uid).array()
        TakeWorkerId -> ByteBuffer.allocate(BYTE_BYTES).put(takeWorkerIdType).array()
        is WorkerId -> ByteBuffer.allocate(BYTE_BYTES + INT_BYTES).put(workerIdType).putInt(o.id).array()
        Reject -> ByteBuffer.allocate(BYTE_BYTES).put(rejectType).array()
        else -> throw InvalidMessageException("Unknown message type ${o.javaClass}")
    }

    override fun fromBinaryJava(bytes: ByteArray, manifest: Class<*>?): Any {
        val b = ByteBuffer.wrap(bytes)
        val type = b.get()
        return when (type) {
            uidRequestType -> UidRequest
            uidResponseType -> UidResponse(b.long)
            takeWorkerIdType -> TakeWorkerId
            workerIdType -> WorkerId(b.int)
            rejectType -> Reject
            else -> throw InvalidMessageException("type=$type")
        }
    }

    override fun includeManifest(): Boolean = false

}

/**
 * 负责维护本节点的[DUidWorker]，在本节点内部提供生成uid服务
 */
class DUidLocalWorkerRouter : UntypedAbstractActor() {

    sealed class Cmd {
        object RegisterWorker : Cmd()
        object RegisterOk : Cmd()
        object UnRegisterWorker : Cmd()
    }

    companion object {
        fun props(dispatcher: String, mailbox: String): Props =
            Props.create(DUidLocalWorkerRouter::class.java)
                .withDispatcher(dispatcher)
                .withMailbox(mailbox)
    }

    private val logger: LoggingAdapter = Logging.getLogger(context.system(), javaClass)

    private val localWorkers: MutableList<ActorRef> = arrayListOf()

    private var nextWorkerIdx = 0

    override fun preStart() {
        logger.info("$self started.")
    }

    override fun postStop() {
        logger.info("$self stopped.")
    }

    override fun onReceive(message: Any) {
        when (message) {
            is UidRequest -> {
                if (localWorkers.isNotEmpty()) {
                    val idx = (nextWorkerIdx + 1).let { if (it >= localWorkers.size) 0 else it }
                    localWorkers[idx].forward(message, context)
                    nextWorkerIdx = idx
                } else {
                    sender.tell(Reject, ActorRef.noSender())
                }
            }
            is Cmd.RegisterWorker -> {
                val s = sender
                if (s !in localWorkers) {
                    localWorkers += s
                    context.watch(s)
                    s.tell(Cmd.RegisterOk, self)
                    logger.info("Add worker $s, workers=$localWorkers")
                }
            }
            is Cmd.UnRegisterWorker -> {
                val s = sender
                if (localWorkers.remove(s)) {
                    context.unwatch(s)
                    logger.info("Remove worker $s, workers=$localWorkers")
                }
            }
            is Terminated -> {
                val s = message.actor
                if (localWorkers.remove(s)) {
                    logger.info("Worker terminated, actor=$s, workers=$localWorkers")
                }
            }
        }
    }
}