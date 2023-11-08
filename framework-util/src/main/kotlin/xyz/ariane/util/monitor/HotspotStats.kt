package xyz.ariane.util.monitor

import xyz.ariane.util.datetime.atDefaultZone
import xyz.ariane.util.dclog.*
import xyz.ariane.util.lang.isLinux
import java.io.File
import java.time.Instant
import java.time.format.DateTimeFormatter
import java.util.*
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.atomic.LongAdder


class EndEvent(
    var taskSignature: String = "n/a",
    var taskCostTimeNanos: Long = 0L
) {
    val costTime: Long get() = EventStat.printTimeUnit.convert(taskCostTimeNanos, TimeUnit.NANOSECONDS)

    fun clear() {
        taskSignature = "n/a"
        taskCostTimeNanos = 0L
    }
}

class HotSpotStatsLog(
    override var clusterId: Long,
    override val processType: String, // 进程类型
    override val processId: Long, // 进程ID
    var throughput: ThroughputStat.Result,
    var qps: ThroughputStat.Result,
    var hot: List<TaskStatsData>,
    var slow: List<TaskStatsData>,
    var call: List<TaskStatsData>,
    var totalJvmMemory: Long,
    var freeJvmMemory: Long,
    var systemLoadAverage: Double,
    var gcs: List<GcStat>,
    var cpuUse: String,
    var total: TaskStatsData,
    _now: Instant = Instant.now()
) : SysLogEntity {
    override val logType: Int = SYSLOG_HOTSPOT_STATS // 日志类型
    override var logId: Long = 0
    override var localDt: String = DateTimeFormatter.ISO_OFFSET_DATE_TIME.format(_now.atDefaultZone())
    override var localTime: Long = _now.toEpochMilli()
}

/**
 * 事件状态处理器
 *
 * 这个实现了Disruptor的onEvent方法，在EndEvent事件到来后做处理。
 */
class EventStat(
    val clusterId: Long,
    val processType: String,
    val processId: Long,
    val dcLogger: IMonitorLogWriter = simpleMonitorLogWriter,
    val minStatsEventNum: Int = DEFAULT_MIN_STATS_EVENT_NUM, // 最小事件数，达到这个事件数就完成一次统计间隔,
    val minPrintPeriodNanos: Long = DEFAULT_PRINT_PERIOD_NANOS // 最小打印间隔,
) {

    private val throughputStat = ThroughputStat() // 吞吐量状态
    private val qpsStat = ThroughputStat() // 每秒请求状态

    /** 总耗时最长排行 */
    private var topTimeConsumers: List<TaskStatsData> = mutableListOf()

    /** 执行最慢，即平均耗时最长的排行 */
    private var topSlowest: List<TaskStatsData> = mutableListOf()

    /** 调用次数最多排行 */
    private var topCalls: List<TaskStatsData> = mutableListOf()

    /** 总时间 */
    private var totalTime: Long = 0

    /** 统计数据 */
    private val statsMap = HashMap<String, TaskStatsData>()

    /** 上一次的总的cpu使用时间 */
    private var lastCpuUseTime = currentCpuUseTime()

    /** 上次输出日志时间 */
    private var lastPrintTime: Long = System.nanoTime()

    /** 自上次输出日志到现在的事件计数 */
    private var eventNumSinceLastPrint: Long = 0

    companion object {
        val printTimeUnit: TimeUnit = TimeUnit.MICROSECONDS
        const val DEFAULT_PRINT_PERIOD_NANOS = 60_000_000_000L // 60s
        const val DEFAULT_MIN_STATS_EVENT_NUM = 1000

        const val topListMaxLength = 100
    }

    fun start() {
        qpsStat.increment()
    }

    fun end(event: EndEvent) {
        throughputStat.increment()

        onEvent(event, 0, true)
    }

    fun onEvent(event: EndEvent, sequence: Long, endOfBatch: Boolean) {
        try {
            addStats(event)
            if (++eventNumSinceLastPrint < minStatsEventNum || !endOfBatch) {
                return
            }

            val now = System.nanoTime()
            if (now - lastPrintTime >= minPrintPeriodNanos) {
                makeTopLists()

                // 开始打印
                val total = printTotalMsgStat()

                val throughput = throughputStat.dumpAndReset(now)
                val qps = qpsStat.dumpAndReset(now)

                val vmStat = VMPerformStats.generate()
                val totalJvmMemory = bytesToMBytes(vmStat.totalJvmMemory)
                val freeJvmMemory = bytesToMBytes(vmStat.freeJvmMemory)
                val systemLoadAverage = vmStat.systemLoadAverage
                val gcs = vmStat.gcs

                val cpuLog = printCpuUseTime()

                val log = HotSpotStatsLog(
                    clusterId,
                    processType,
                    processId,
                    throughput,
                    qps,
                    topTimeConsumers,
                    topSlowest,
                    topCalls,
                    totalJvmMemory,
                    freeJvmMemory,
                    systemLoadAverage,
                    gcs,
                    cpuLog,
                    total
                )

                dcLogger.write(log)

                // 重置
                lastPrintTime = now
                eventNumSinceLastPrint = 0L
                totalTime = 0L
                statsMap.clear()
            }
        } catch (e: Throwable) {
//            dcLogger.writeDc(e)
        } finally {
            event.clear()
        }
    }
//
//    private fun printOtherStats(currentNanos: Long) {
//        throughputStat.dumpAndReset(currentNanos).let { it ->
//            log.throughput = it
////            dcLogger.writeDc(HotspotStatsStrLog(clusterId, ">>>>>>>>> THROUGHPUT: $average, PERIOD: ${periodSeconds}s"))
//        }
//        qpsStat.dumpAndReset(currentNanos).let { it->
//            log.qps = it
////            dcLogger.writeDc(">>>>>>>>> QPS: $average, PERIOD: ${periodSeconds}s")
//        }
//    }

//    private fun printTopListsToLogger(log: HotSpotStatsLog) {
//        log.hot = topTimeConsumers
//        log.slow = topSlowest
//        log.call = topCalls
////        dcLogger.writeDc("====================== 总体最热 top {} ======================", HotspotStats.topListMaxLength)
////        printTable(topTimeConsumers, "hot")
////        dcLogger.writeDc("====================== 平均最慢 top {} ======================", HotspotStats.topListMaxLength)
////        printTable(topSlowest, "slow")
////        dcLogger.writeDc("====================== 次数最多 top {} ======================", HotspotStats.topListMaxLength)
////        printTable(topCalls, "count")
//    }

//    private fun printTable(topList: List<TaskStatsData>?, tag: String = "") {
//        topList?.forEach {
//            val info = String.format(
//                tagPrintFormat,
//                tag,
//                it.selfTotalTime,
//                it.selfTotalTimeRate / 100.0,
//                it.averageCallTime,
//                it.callTimes,
//                it.count0T9,
//                it.count10T99,
//                it.count100T999,
//                it.count1kT10k,
//                it.count10kT100k,
//                it.count100kTmax,
//                it.taskSignature
//            )
//            dcLogger.writeDc(info)
//        }
//    }

    private fun makeTopLists() {
        val sortList = ArrayList(statsMap.values)
        for (statsData in sortList) {
            statsData.selfTotalTimeRate = (statsData.selfTotalTime * 10000 / totalTime).toInt()
        }
        val topListLength = Math.min(topListMaxLength, sortList.size) // 计算限值

        sortList.sortByDescending { it.selfTotalTime }
        topTimeConsumers = ArrayList(sortList.subList(0, topListLength))

        sortList.sortByDescending { it.averageCallTime }
        topSlowest = ArrayList(sortList.subList(0, topListLength))

        sortList.sortByDescending { it.callTimes }
        topCalls = ArrayList(sortList.subList(0, topListLength))
    }

    private fun addStats(event: EndEvent) {
        totalTime += event.costTime
        if (totalTime > 0) {
            var signature = event.taskSignature
            if (signature.startsWith("execution")) {
                signature = signature.substring(signature.indexOf("(") + 1, signature.length - 1)
            }

            // 准备stats数据
            var statsData: TaskStatsData? = statsMap[signature]
            if (statsData == null) {
                statsData = TaskStatsData(signature)
                statsMap[signature] = statsData
            }
            statsData.selfTotalTime += event.costTime
            statsData.callTimes++
            statsData.averageCallTime = (statsData.selfTotalTime / statsData.callTimes).toFloat()
            when (event.costTime) {
                in 0..9 -> statsData.count0T9++
                in 10..99 -> statsData.count10T99++
                in 100..999 -> statsData.count100T999++
                in 1000..9999 -> statsData.count1kT10k++
                in 10000..99999 -> statsData.count10kT100k++
                else -> statsData.count100kTmax++
            }
        }
    }

    private fun bytesToMBytes(bytes: Long): Long = bytes / (1024L * 1024L)

    private fun printTotalMsgStat(): TaskStatsData {
        val total = TaskStatsData("TOTAL")
        total.selfTotalTime = totalTime
        total.selfTotalTimeRate = 10000
        for (statData in statsMap.values) {
            total.callTimes += statData.callTimes
            total.count0T9 += statData.count0T9
            total.count10T99 += statData.count10T99
            total.count100T999 += statData.count100T999
            total.count1kT10k += statData.count1kT10k
            total.count10kT100k += statData.count10kT100k
            total.count100kTmax += statData.count100kTmax
        }
        total.averageCallTime = (total.selfTotalTime / total.callTimes).toFloat()

        return total
//        log.total = total
//        printTable(listOf(total), "TOTAL")
    }

//    private fun printCurrentVMStat(log: HotSpotStatsLog) {
//        val vmStat = VMPerformStats.generate()
//
//        log.totalJvmMemory = bytesToMBytes(vmStat.totalJvmMemory)
//        log.freeJvmMemory = bytesToMBytes(vmStat.freeJvmMemory)
//        log.systemLoadAverage = vmStat.systemLoadAverage
//        log.gcs = vmStat.gcs
//
////        dcLogger.writeDc(
////            ">>>>>>>>> TotalJvmMemory: {}m, FreeJvmMemory: {}m, SystemLoadAverage：{}",
////            bytesToMBytes(vmStat.totalJvmMemory), bytesToMBytes(vmStat.freeJvmMemory), vmStat.systemLoadAverage
////        )
////        for (gc in vmStat.gcs) {
////            dcLogger.writeDc("GcName:${gc.gcName}，Count：${gc.gcCount}，CostTime：${gc.gcTime}")
////        }
//    }

    private fun printCpuUseTime(): String {
        if (!isLinux()) {
            return ""
        }

        var cpuLog = ""
        try {
            val currentCpuUseTime = currentCpuUseTime()
            val isOk = !currentCpuUseTime.isEmpty() && !lastCpuUseTime.isEmpty()
//            dcLogger.writeDc("====================== 时间段CPU使用率  {} ======================", isOk)
            if (isOk) {
                val midList = arrayListOf<Long>()
                var allTime: Long = 0
                for (i in currentCpuUseTime.indices) {
                    val mid = currentCpuUseTime[i] - lastCpuUseTime[i]
                    midList.add(mid)
                    allTime += mid
                }
                val total = allTime / 100.0

                // 3.x内核有10个字段，这里只显示8个字段
                cpuLog = String.format(
                    "cpu: %4.2f%%us %4.2f%%ni %4.2f%%sy %4.2f%%id %4.2f%%wa %4.2f%%hi %4.2f%%si %4.2f%%st",
                    midList[0] / total, midList[1] / total, midList[2] / total, midList[3] / total,
                    midList[4] / total, midList[5] / total, midList[6] / total, midList[7] / total
                )
            } else {
                cpuLog = ""
            }

            lastCpuUseTime = currentCpuUseTime

        } catch (e: Exception) {
//            dcLogger.writeDc("printCpuUseTime:", e)
        }

        return cpuLog
    }

    private fun currentCpuUseTime(): List<Long> {
        if (!isLinux()) {
            return emptyList()
        }
        val timeList = arrayListOf<Long>()
        val file = File("/proc/stat")
        try {
            file.bufferedReader().use { br ->
                // 只取第一行，第一行为linux启动至现在 总的cpu使用时间
                val line = br.readLine()
                if (line != null) {
                    val token = StringTokenizer(line)
                    if ("cpu" == token.nextToken()) {
                        while (token.hasMoreElements()) {
                            val time = token.nextToken().toLong()
                            timeList.add(time)
                        }
                    }
                }
            }
        } catch (e: Exception) {
//            dcLogger.writeDc("HotspotStatsImpl：", e)
        }

        return timeList
    }
}

/**
 * 统计吞吐量
 */
class ThroughputStat {

    data class Result(val periodSeconds: Long, val average: Long)

    interface Counter {
        fun increment()
        fun reset()
        fun count(): Long
    }

    class LongAdderCounter : Counter {

        private val l = LongAdder()

        override fun increment() {
            l.increment()
        }

        override fun reset() {
            l.reset()
        }

        override fun count(): Long = l.sum()
    }

    class AtomicLongCounter : Counter {

        private val l = AtomicLong()

        override fun increment() {
            l.incrementAndGet()
        }

        override fun reset() {
            l.set(0)
        }

        override fun count(): Long = l.toLong()
    }

    /** 统计开始时间 */
    private var startNanos = System.nanoTime()

    /** 请求计数 */
    private val counter: Counter = LongAdderCounter()

    /** Will be invoked in multi threads */
    fun increment() {
        counter.increment()
    }

    private fun reset() {
        startNanos = System.nanoTime()
        counter.reset()
    }

    /** Will be invoked in single thread */
    fun dumpAndReset(currentNanos: Long): Result {
        val periodSeconds = TimeUnit.NANOSECONDS.toSeconds(currentNanos - startNanos)
        val r = Result(
            periodSeconds = periodSeconds,
            average = if (periodSeconds > 0L) counter.count() / periodSeconds else -1L
        )

        // 重置
        reset()
        return r
    }
}

/**
 * 任务状态数据
 */
class TaskStatsData(val taskSignature: String) {
    /** 自用总时间（毫秒） */
    var selfTotalTime: Long = 0L

    /** 自用时间占总时间的比例(万分比) */
    var selfTotalTimeRate = 0

    /** 调用总次数 */
    var callTimes: Long = 0L

    /** 平均调用时间 */
    var averageCallTime = 0F

    var count0T9 = 0L
    var count10T99 = 0L
    var count100T999 = 0L
    var count1kT10k = 0L
    var count10kT100k = 0L
    var count100kTmax = 0L
}