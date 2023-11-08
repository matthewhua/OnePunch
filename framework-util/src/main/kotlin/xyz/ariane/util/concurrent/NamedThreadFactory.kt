package xyz.ariane.util.concurrent

import java.util.concurrent.ThreadFactory
import java.util.concurrent.atomic.AtomicInteger

/**
 * 命名的线程池
 */
class NamedThreadFactory @JvmOverloads constructor(
    private val namePrefix: String,
    private val isDaemon: Boolean = false
) : ThreadFactory {

    private val group: ThreadGroup = System.getSecurityManager()?.threadGroup ?: Thread.currentThread().threadGroup

    private val threadNumber = AtomicInteger(1)

    override fun newThread(r: Runnable): Thread =
        Thread(group, r, "$namePrefix-${threadNumber.getAndIncrement()}", 0).also {
            if (it.isDaemon != isDaemon) {
                it.isDaemon = isDaemon
            }
            if (it.priority != Thread.NORM_PRIORITY) {
                it.priority = Thread.NORM_PRIORITY
            }
        }
}
