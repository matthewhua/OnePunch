package xyz.ariane.util.dclog

import com.fasterxml.jackson.annotation.JsonFilter
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.ser.impl.SimpleBeanPropertyFilter
import com.fasterxml.jackson.databind.ser.impl.SimpleFilterProvider
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper

// 下面是framework内定的系统日志类型（1~100）
// 上层模块定义日志常量时需要从101开始
const val SYSLOG_HOTSPOT_STATS = 1
const val SYSLOG_WDB_STATS = 2

@JsonFilter("consoleLogFilter")
class ConsoleLogFilter

val consoleLogMapper: ObjectMapper = jacksonObjectMapper()
    .addMixIn(LogEntity::class.java, ConsoleLogFilter::class.java)
    .setFilterProvider(
        SimpleFilterProvider().addFilter(
            "consoleLogFilter",
            SimpleBeanPropertyFilter.serializeAllExcept("logId", "localTime", "clusterId", "ecpMsg")
        )
    )


interface SysLogEntity : LogEntity {
    val logType: Int // 日志类型
    val processType: String // 进程类型
    val processId: Long // 进程ID
}

// 日志条目接口（不要直接继承）
interface LogEntity {
    var logId: Long // 日志的唯一ID
    var localDt: String // 当前时间
    var localTime: Long // 当前时间戳
    var clusterId: Long // 集群ID

    fun toConsoleJson(): String {
        return consoleLogMapper.writeValueAsString(this)
    }
}
