package xyz.ariane.util.lang

import akka.actor.ActorRef
import akka.actor.NotInfluenceReceiveTimeout
import akka.event.LoggingAdapter
import org.slf4j.Logger
import xyz.ariane.util.akka.tellNoSender
import xyz.ariane.util.monitor.EndEvent
import java.time.Clock
import java.time.Duration
import java.time.Instant

fun encodeThrowable(e: Throwable): RuntimeException = when (e) {
    is RuntimeException -> e
    else -> RuntimeException(e)
}

/** Kotlin API */
inline fun tryCatch(logger: LoggingAdapter, func: () -> Unit) {
    try {
        func()
    } catch (e: Throwable) {
        logger.error(e, "")
    }
}

/** Kotlin API */
inline fun tryCatch(logger: Logger, func: () -> Unit) {
    try {
        func()
    } catch (e: Throwable) {
        logger.error("", e)
    }
}

/** Java API */
fun tryCatch(logger: LoggingAdapter, runnable: Runnable) {
    try {
        runnable.run()
    } catch (e: Throwable) {
        logger.error(e, "")
    }
}

/** Java API */
fun tryCatch(logger: Logger, runnable: Runnable) {
    try {
        runnable.run()
    } catch (e: Throwable) {
        logger.error("", e)
    }
}

abstract class InitializeRequired<in D> {

    var initialized: Boolean = false
        private set

    fun initialize(initData: D) {
        require(!initialized) { "只能初始化一次" }
        initializeImpl(initData)
        initialized = true
    }

    protected abstract fun initializeImpl(initData: D)

}

open class NamedRunnable(val name: String, val func: () -> Unit) : Runnable, NotInfluenceReceiveTimeout {

    init {
        require(!name.isBlank()) { "名字不能为空" }
    }

    override fun run() {
        func()
    }

    override fun toString(): String {
        return "NamedRunnable($name)"
    }
}

/**
 * 表示tick触发的任务
 */
class TickTask(name: String, func: () -> Unit) : NamedRunnable(name, func)

/**
 * 基于tick次数的计时器，每tick[cycle]次，[timeUp]返回true
 *
 * @param cycle time up的tick周期次数
 * @param firstTimeUp 第一次time up的tick次数
 */
class TickTimer(private val cycle: Int, firstTimeUp: Int = 0) {
    init {
        require(cycle > 0) { "Invalid cycle $cycle" }
        require(firstTimeUp >= 0) { "Invalid initialCycle $firstTimeUp" }
    }

    private var tick: Int = -firstTimeUp

    val timeUp: Boolean get() = tick == 0

    fun tick() {
        ++tick
        if (tick >= cycle) {
            tick = 0
        }
    }

    inline fun whenTimeUp(func: () -> Unit) {
        tick()
        if (timeUp) {
            func()
        }
    }

}

/**
 * 基于tick的，固定时间间隔执行任务的定时器
 */
class DurationTickTimer @JvmOverloads constructor(
    /** 执行任务时间间隔 */
    val timeUpInterval: Duration,
    /** 任务 */
    val task: NamedRunnable,
    /** 时钟 */
    val clock: Clock = Clock.systemDefaultZone()
) {

    init {
        require(!timeUpInterval.isNegative && !timeUpInterval.isZero) {
            "Invalid timeUpInterval:$timeUpInterval"
        }
    }

    /** 下次执行任务时间 */
    private var nextExecTime: Instant = clock.instant().plus(timeUpInterval)

    @JvmOverloads
    fun tick(now: Instant = clock.instant(), exec: (NamedRunnable) -> Unit = { it.run() }) {
        if (now.isAfter(nextExecTime)) {
            exec(task)
            nextExecTime = now.plus(timeUpInterval)
        }
    }

}

/**
 * 定时检查对象变化
 *
 */
abstract class TickCheckChangeTracker<T> @JvmOverloads constructor(
    checkInterval: Duration,
    clock: Clock = Clock.systemDefaultZone()
) {

    private var value: T? = null

    private val timer = DurationTickTimer(
        timeUpInterval = checkInterval,
        task = NamedRunnable("TickCheckChange") { check() },
        clock = clock
    )

    private fun check() {
        val newValue = getCurrentValue()
        if (newValue != value) {
            value = newValue
            onChanged(newValue)
        }
    }

    fun tick() {
        timer.tick()
    }

    protected abstract fun getCurrentValue(): T

    protected abstract fun onChanged(value: T)

}

inline fun <R> profile(needMonitor: Boolean, hotspotStatsActor: ActorRef, signature: String, func: () -> R): R {
    return if (needMonitor) {
        val t0 = System.nanoTime()
        hotspotStatsActor.tellNoSender("start")

        // 执行
        val r = func()

        val nanoCost = System.nanoTime() - t0
        val costTimeNanos = Math.max(0L, nanoCost)
        hotspotStatsActor.tellNoSender(EndEvent(signature, costTimeNanos))

        r

    } else {
        func()
    }
}

