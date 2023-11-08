package xyz.ariane.util.dclog

interface IMonitorLogWriter {

    /**
     * 记录性能和系统状态日志
     */
    fun write(log: LogEntity)

}