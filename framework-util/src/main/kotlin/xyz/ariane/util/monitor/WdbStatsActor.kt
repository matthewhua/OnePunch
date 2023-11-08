package xyz.ariane.util.monitor

import akka.actor.AbstractActor
import akka.actor.Props
import com.fasterxml.jackson.annotation.JsonIgnore
import xyz.ariane.util.datetime.atDefaultZone
import xyz.ariane.util.dclog.IMonitorLogWriter
import xyz.ariane.util.dclog.SYSLOG_WDB_STATS
import xyz.ariane.util.dclog.SysLogEntity
import xyz.ariane.util.memodb.DirtyCheckRt
import xyz.ariane.util.memodb.emptyDirtyCheckRt
import java.time.Clock
import java.time.Instant
import java.time.format.DateTimeFormatter


// 每类元素的延迟信息统计
data class WdbStatLog4CheckEntity(
    val elementNum: Int,
    val count: Int
)

// 数据库检测信息统计
data class WdbStatLog4Check(
    val title: String,
    val checkInfos: MutableList<WdbStatLog4CheckEntity> = mutableListOf()
)

// 每种类的延迟信息统计
data class WdbStatLog4DelayEntity(
    val className: String,
    val totalNum: Int,
    val averageDelay: Long,
    val delayInfo: String
)

// 数据库延迟信息统计
data class WdbStatLog4Delay(
    val title: String,
    val infos: MutableList<WdbStatLog4DelayEntity> = mutableListOf()
)

/**
 * 数据状态性能日志
 */
class WdbStatLog(
    override var clusterId: Long,
    override val processType: String, // 进程类型
    override val processId: Long, // 进程ID
    var wdbType: Int, // 1：World、2：Home、3：Pub、4：Login
    var wdbAvgTracker: Int, // 每轮检测的tracker的平均数量，也反映了当前内存中总共多少tracker。
    var wdbAvgCheck: Int, // 每轮检测的平均数量
    var wdbCheckNormal: Int, // 提前结束数量
    var wdbCheckOutTime: Int, // 超时结束数量
    var wdbCheckOutLoop: Int, // 超循环结束数量
    @JsonIgnore
    val trackerInfo: WdbStatLog4Check, // 跟踪信息
    @JsonIgnore
    val checkInfo: WdbStatLog4Check,
    @JsonIgnore
    val delayCosts: WdbStatLog4Delay,
    _now: Instant = Instant.now()
) : SysLogEntity {
    override val logType: Int = SYSLOG_WDB_STATS
    override var logId: Long = 0
    override var localDt: String = DateTimeFormatter.ISO_OFFSET_DATE_TIME.format(_now.atDefaultZone())
    override var localTime: Long = _now.toEpochMilli()
}

/**
 * Disruptor事件对象
 */
data class TickEvent(
    var dirtyCheckRt: DirtyCheckRt = emptyDirtyCheckRt,
    var delayInfoList: List<DelayInfo> = emptyList(),
) {
    fun clear() {
        dirtyCheckRt = emptyDirtyCheckRt
        delayInfoList = emptyList()
    }
}

class DelayStats(
    var simpleClassName: String,
    var totalDelay: Long = 0, // SS：总延迟
    val histogram: IntArray = IntArray(6), // SS：各个延迟分段的统计数量，每个分段差一个数量级，比如1ms和10ms和100ms。
    var averageDelay: Long = -1L, // SS：平均延迟
    var totalNum: Int = 0 // SS：总数
) {
    fun updateAverageDelay() {
        totalNum = histogram.sum()
        if (totalNum > 0L) {
            averageDelay = totalDelay / totalNum
        }
    }
}

/**
 * 特定数据类的延迟信息
 */
data class DelayInfo(val className: String, val delayMillis: Long)

interface WdbStatsReporter {
    fun report(event: TickEvent)
}

class WdbStatsActor(
    val clusterId: Long,
    val processType: String,
    val processId: Long,
    val clock: Clock,
    val dcLogger: IMonitorLogWriter
) : AbstractActor() {

    val handler = WdbEventStatHandler(clusterId, processType, processId, clock, dcLogger)

    companion object {
        fun props(
            clusterId: Long,
            processType: String,
            processId: Long,
            clock: Clock,
            dcLogger: IMonitorLogWriter,
            dispatcher: String,
            mailBox: String
        ): Props {
            return Props.create(WdbStatsActor::class.java) {
                WdbStatsActor(clusterId, processType, processId, clock, dcLogger)
            }
                .withDispatcher(dispatcher)
                .withMailbox(mailBox)
        }
    }

    override fun createReceive(): Receive {
        return receiveBuilder()
            .match(TickEvent::class.java) { value ->
                handler.onEvent(value, 0, true)
            }.build()
    }

}