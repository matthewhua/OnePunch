package xyz.ariane.util.dclog

import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.ariane.util.json.mapperUsedInGame

val simpleFileLogWriter: SimpleFileLogWriter = SimpleFileLogWriter()

class SimpleFileLogWriter : ILogWriter {

    private val logger: Logger = LoggerFactory.getLogger(javaClass)

    override fun writeDc(log: LogEntity) {
        logger.info(mapperUsedInGame.writeValueAsString(log))
    }
}