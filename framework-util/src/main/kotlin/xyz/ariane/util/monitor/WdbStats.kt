package xyz.ariane.util.monitor

import com.google.common.collect.HashMultiset
import xyz.ariane.util.dclog.*
import xyz.ariane.util.memodb.DirtyCheckResult
import java.time.Clock
import java.util.*
import java.util.concurrent.TimeUnit

/**
 * 数据库状态的事件处理
 */
class WdbEventStatHandler(
    val clusterId: Long,
    val processType: String,
    val processId: Long,
    val clock: Clock,
    val dcLogger: IMonitorLogWriter
) {

    private var lastPrintTime: Long = clock.millis() // SS：上次打印时间

    private var accumulativeTimes = 0 // 记录次数

    // SS：总数分布，100的度量范围
    private var accumulativeTrackerNum: Long = 0L // 记录事件时累加上去的tracker量
    private val totalNumHistogramCounter = HashMultiset.create<Int>()

    // SS：检测数分布，10的度量范围
    private var accumulativeCheckNum: Long = 0L // 记录事件时累加上去的check量
    private val checkedNumHistogramCounter = HashMultiset.create<Int>()

    // SS：根据类名维护的延迟状态表
    private val delayStatsMap = HashMap<String, DelayStats>(100)

    // SS：脏检测结束情况分布。超循环说明检测数太多，超时说明hash计算太慢。
    private val dirtyCheckRtStat = EnumMap<DirtyCheckResult, Int>(DirtyCheckResult::class.java)

    companion object {
        const val TOTAL_NUM_SLOT_SCALE = 100
        const val CHECKED_NUM_SLOT_SCALE = 10
        const val HISTOGRAM_MAX_BAR_LENGTH = 80

        val PRINT_INTERVAL_MILLIS: Long = TimeUnit.MINUTES.toMillis(5L) // 打印输出的间隔
    }

    fun onEvent(event: TickEvent, sequence: Long, endOfBatch: Boolean) {
        val dirtyCheckRt = event.dirtyCheckRt
        val delayInfos = event.delayInfoList

        accumulativeTimes += 1

        // 提取次数相关的信息
        accumulativeTrackerNum += dirtyCheckRt.totalNum
        totalNumHistogramCounter.add(dirtyCheckRt.totalNum / TOTAL_NUM_SLOT_SCALE) // 这里面存的数是除以过TOTAL_NUM_SLOT_SCALE的
        accumulativeCheckNum += dirtyCheckRt.checkedNum
        checkedNumHistogramCounter.add(dirtyCheckRt.checkedNum / CHECKED_NUM_SLOT_SCALE)

        val dirtyFinishRt = dirtyCheckRt.dirtyFinishRt
        val rtNum = dirtyCheckRtStat[dirtyFinishRt] ?: 0
        dirtyCheckRtStat[dirtyFinishRt] = rtNum + 1

        // 提取延迟相关的信息
        for ((className, delay) in delayInfos) {
            if (delay >= 0L) {
                val s = delayStatsMap.getOrPut(className) {
                    DelayStats(className.substring(className.lastIndexOf(".") + 1))
                }
                s.totalDelay += delay // SS：总延迟？有意义吗？
                val histogram = s.histogram
                when (delay) {
                    in 0L until 10L -> ++histogram[0] // 10ms以内
                    in 10L until 100L -> ++histogram[1] // 10ms 到 100ms 之间
                    in 100L until 1000L -> ++histogram[2] // 100ms 到 1s 之间
                    in 1000L until 10_000L -> ++histogram[3] // 1s 到 10s 之间！
                    in 10_000L until 100_000L -> ++histogram[4] // 10s 到 100s 之间！
                    else -> ++histogram[5] // 超过 100s 了！
                }
            }
        }

        event.clear()

        if (!endOfBatch) {
            return
        }

        val now = clock.millis()
        if (now - lastPrintTime < PRINT_INTERVAL_MILLIS) {
            return
        }

        // 输出日志
        logStats(dirtyCheckRt.wdbType)

        // 重置
        resetStats(now)
    }

    /**
     * 输出统计日志
     */
    private fun logStats(wdbType: Int) {
        val avgTrackerNum = (accumulativeTrackerNum / accumulativeTimes).toInt()
        val avgCheckNum = (accumulativeCheckNum / accumulativeTimes).toInt()
        val wdbLog = WdbStatLog(
            clusterId,
            processType,
            processId,
            wdbType,
            avgTrackerNum, avgCheckNum,
            0, 0, 0,
            WdbStatLog4Check("跟踪的总对象数分布（按100的区间来间隔）"),
            WdbStatLog4Check("心跳检查对象数分布（按10的区间来间隔）"),
            WdbStatLog4Delay("EntityWrapper检查延迟统计")
        )

        // SS：显示脏检测结束情况分布
        wdbLog.wdbCheckNormal = dirtyCheckRtStat[DirtyCheckResult.DIRTY_CHECK_NO_MORE] ?: 0
        wdbLog.wdbCheckOutTime = dirtyCheckRtStat[DirtyCheckResult.DIRTY_CHECK_OUT_TIME] ?: 0
        wdbLog.wdbCheckOutLoop = dirtyCheckRtStat[DirtyCheckResult.DIRTY_CHECK_OUT_LOOP] ?: 0

        // SS：显示统计图：包含的总对象数分布
        // SS：这个图显示了在打印间隔内，trackerMap中的对象数分布。
        // SS：随着开服时间的增加，trackerMap中的数量会不断增长！
        drawHistogram(
            wdbLog.trackerInfo, "跟踪的总对象数分布（按100的区间来间隔）", totalNumHistogramCounter,
            TOTAL_NUM_SLOT_SCALE
        )

        // SS：显示统计图：心跳检查对象数分布
        // SS：每个心跳周期check的tracker数量分布，如果太少，那么可能是变更比较少或者是CPU压力太大以致于在检测周期内来不及Check。
        drawHistogram(
            wdbLog.checkInfo, "心跳检查对象数分布（按10的区间来间隔）", checkedNumHistogramCounter,
            CHECKED_NUM_SLOT_SCALE
        )

        // SS：显示统计图：EntityWrapper检查延迟统计
        // SS：统计不同的类的检测延迟。
        // SS：来不及对变更进行Check的情况，会反应到检测延迟上来，超过1s的数量如果太多，并且有进入10s的区间的，就肯定出问题了！
        // EntityWrapper检查延迟统计
        for ((_, delayStats) in delayStatsMap) {
            delayStats.updateAverageDelay() // SS：计算每个类对应的平均延迟
        }

        val delayCosts = wdbLog.delayCosts
        delayStatsMap.values
            .sortedByDescending { it.totalNum }
            .forEach { delayStats ->
                delayCosts.infos.add(
                    WdbStatLog4DelayEntity(
                        delayStats.simpleClassName,
                        delayStats.totalNum, // 更新总数
                        delayStats.averageDelay, // 平均延迟
                        Arrays.toString(delayStats.histogram)
                    )
                )
            }

        dcLogger.write(wdbLog)
    }

    private fun resetStats(now: Long) {
        lastPrintTime = now
        accumulativeTimes = 0
        accumulativeTrackerNum = 0
        totalNumHistogramCounter.clear()
        accumulativeCheckNum = 0
        checkedNumHistogramCounter.clear()
        dirtyCheckRtStat.clear()
        delayStatsMap.clear()
    }

    private fun drawHistogram(log: WdbStatLog4Check, title: String, counter: HashMultiset<Int>, scale: Int) {
        val sortedEntries = counter.entrySet().sortedBy { it.element }
        sortedEntries.forEach {
            log.checkInfos.add(WdbStatLog4CheckEntity(it.element * scale, it.count))
        }
    }
}