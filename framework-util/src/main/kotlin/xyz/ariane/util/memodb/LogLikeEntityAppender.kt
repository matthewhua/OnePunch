package xyz.ariane.util.memodb

import com.google.common.collect.ImmutableList
import org.slf4j.Logger
import xyz.ariane.util.concurrent.ActorCompletionStage
import xyz.ariane.util.dclog.lzDebug
import java.util.*

/**
 * 用于保存日志类(只会save)数据，失败或超时时重试，保证至少一次成功
 *
 * [append]的[IEntity]对象会被缓冲，需要使用者调用[flush]开始批量执行save操作
 *
 */
class LogLikeEntityAppender(
    private val createACS: () -> ActorCompletionStage<Unit>,
    private val fetchDao: () -> CommonDao,
    private val logger: Logger,
    private val batchSize: Int,
    private val maxCapacity: Long = 1000L
) {

    private val queue: ArrayDeque<IEntity> = ArrayDeque(5)

    val queueSize: Int get() = queue.size

    var flushing: Boolean = false
        private set

    fun isClean(): Boolean = queue.isEmpty()

    /**
     * 附加日志
     */
    fun append(entity: IEntity) {
        if (queue.size >= maxCapacity) {
            logger.warn("Q is full, drop [{}]", entity)
            return
        }
        queue.offer(entity)
    }

    /**
     * 刷入日志
     */
    fun flush() {
        if (flushing) {
            return
        } else if (queue.isEmpty()) {
            return
        }

        // 标记刷入中
        flushing = true

        // 得到队列中所有的entities
        val allEntities = ImmutableList.copyOf(queue)
        val dao = fetchDao()
        createACS().supplyIoKt {
            val bs = batchSize
            dao.execWithTransaction { session ->
                allEntities.forEachIndexed { i, entity ->
                    session.save(entity)
                    session.flushIfReachBatchSize(i, bs)
                }
            }
        }.whenCompleteKt { _, err ->
            try {
                if (err != null) {
                    // TODO 如何处理无法恢复的失败？如主键重复等，重试也不会成功
                    logger.error("Append failed, will retry.", err)
                } else {
                    // 成功了
                    repeat(allEntities.size) { queue.pop() }

                    logger.lzDebug { "${allEntities.size} entities append succeeded, ${queue.size} left." }
                }
            } catch (e: Exception) {
                logger.error("", e)
            } finally {
                flushing = false
            }
        }
    }

}