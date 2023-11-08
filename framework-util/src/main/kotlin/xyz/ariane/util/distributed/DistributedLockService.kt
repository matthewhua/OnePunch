package xyz.ariane.util.distributed

import akka.actor.*
import akka.cluster.singleton.ClusterSingletonManager
import akka.cluster.singleton.ClusterSingletonManagerSettings
import akka.event.Logging
import akka.event.LoggingAdapter
import akka.serialization.JSerializer
import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import com.google.common.cache.LoadingCache
import xyz.ariane.util.distributed.DLockMessage.*
import scala.concurrent.duration.FiniteDuration
import java.nio.ByteBuffer
import java.util.concurrent.TimeUnit

const val HANDOFF_WAIT_SECONDS = 3L

fun startClusterLockManager(
    actorSystem: ActorSystem,
    settings: ClusterSingletonManagerSettings,
    name: String,
    dispatcher: String
) {

    val props = ClusterLockManager.props(name, dispatcher)
    actorSystem.actorOf(ClusterSingletonManager.props(props, HandOver, settings), name)
}

/**
 * 集群锁管理器，作为集群单实例集中管理所对象
 *
 * 为避免单点性能瓶颈，每一类锁应该创建单独的锁管理器，而不要共享同一个
 *
 * ** 注意 ** 锁数据没有做持久化，在跨节点迁移时锁数据会被清空，可能造成问题。
 * 为此在节点迁移时会先等待[HANDOFF_WAIT_SECONDS]秒，尽量等待已经获取到锁的事务完成，在进行迁移。
 * 等待期间所有获取锁的请求都会返回失败。
 *
 */
class ClusterLockManager(val name: String) : UntypedAbstractActor() {

    companion object {
        fun props(name: String, dispatcher: String): Props {
            return Props.create(ClusterLockManager::class.java) { ClusterLockManager(name) }.withDispatcher(dispatcher)
        }
    }

    enum class State {
        /** 正常工作 */
        UP,
        /** 正在跨节点迁移 */
        HANDOVER,
    }

    var state: State = State.UP

    val logger: LoggingAdapter = Logging.getLogger(context.system(), ClusterLockManager::class.java)

    /** 锁缓存 */
    val lockTable: LoadingCache<String, String> = CacheBuilder.newBuilder()
        .concurrencyLevel(1)
        .expireAfterAccess(1, TimeUnit.HOURS)
        .build(CacheLoader.from { key: String? -> key })

    override fun preStart() {
        logger.info("$self started")
    }

    override fun postStop() {
        logger.info("$self stopped")
    }

    override fun onReceive(msg: Any) {
        when (state) {
            State.UP -> when (msg) {
                is TryLock -> tryLock(msg)
                is Unlock -> lockTable.invalidate(msg.key)
                is HandOver -> enterHandoverState()
                else -> unhandled(msg)
            }
            State.HANDOVER -> sender.tell(LockFailed, ActorRef.noSender())
        }
    }

    private fun enterHandoverState() {
        state = State.HANDOVER
        val waitDuration = FiniteDuration(HANDOFF_WAIT_SECONDS + 1, TimeUnit.SECONDS)
        context.system().scheduler()
            .scheduleOnce(waitDuration, self, PoisonPill.getInstance(), context.dispatcher(), self)
    }

    private fun tryLock(message: TryLock) {
        val key = message.key
        if (lockTable.asMap().containsKey(key)) {
            sender.tell(LockFailed, ActorRef.noSender())
        } else {
            lockTable.get(key)
            sender.tell(LockSucceeded, ActorRef.noSender())
        }
    }

}

sealed class DLockMessage {
    class TryLock(val key: String) : DLockMessage()

    object LockSucceeded : DLockMessage()

    object LockFailed : DLockMessage()

    class Unlock(val key: String) : DLockMessage()

    object HandOver : DLockMessage()
}

class DLockMessageSerializer : JSerializer() {

    private val tryLockType: Byte = 1
    private val lockSucceededType: Byte = 2
    private val lockFailedType: Byte = 3
    private val unlockType: Byte = 4
    private val handOverType: Byte = 5

    override fun fromBinaryJava(bytes: ByteArray, manifest: Class<*>?): Any {
        val buffer = ByteBuffer.wrap(bytes)
        val type = buffer.get()
        return when (type) {
            tryLockType -> TryLock(readKey(buffer))
            unlockType -> Unlock(readKey(buffer))
            lockSucceededType -> LockSucceeded
            lockFailedType -> LockFailed
            handOverType -> HandOver
            else -> throw InvalidMessageException("Unknown message type $type")
        }
    }

    private fun readKey(buffer: ByteBuffer): String {
        val keyBuffer = ByteArray(buffer.remaining())
        buffer.get(keyBuffer)
        return keyBuffer.toString(Charsets.UTF_8)
    }

    override fun identifier(): Int = 20160213

    override fun toBinary(o: Any): ByteArray = when (o) {
        is TryLock -> {
            val bytes = o.key.toByteArray()
            ByteBuffer.allocate(1 + bytes.size).put(tryLockType).put(bytes).array()
        }
        is Unlock -> {
            val bytes = o.key.toByteArray()
            ByteBuffer.allocate(1 + bytes.size).put(unlockType).put(bytes).array()
        }
        LockSucceeded -> ByteBuffer.allocate(1).put(lockSucceededType).array()
        LockFailed -> ByteBuffer.allocate(1).put(lockFailedType).array()
        HandOver -> ByteBuffer.allocate(1).put(handOverType).array()
        else -> throw InvalidMessageException("Unknown message type ${o.javaClass}")
    }

    override fun includeManifest(): Boolean = false
}

