package xyz.ariane.util.dclog

interface ILogWriter {
    /**
     * 记录游戏日志
     */
    fun writeDc(log: LogEntity)
}