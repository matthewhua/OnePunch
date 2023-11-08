package xyz.ariane.util.dclog

import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.ariane.util.json.mapperUsedInGame

val simpleMonitorLogWriter: SimpleMonitorLogWriter = SimpleMonitorLogWriter()

class SimpleMonitorLogWriter: IMonitorLogWriter {

    private val logger: Logger = LoggerFactory.getLogger(javaClass)

    override fun write(log: LogEntity) {
        logger.info(mapperUsedInGame.writeValueAsString(log))
    }
}