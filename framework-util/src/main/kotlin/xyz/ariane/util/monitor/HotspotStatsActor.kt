package xyz.ariane.util.monitor

import akka.actor.AbstractActor
import akka.actor.Props
import xyz.ariane.util.dclog.IMonitorLogWriter

class HotspotStatsActor(
    val clusterId: Long,
    val processType: String,
    val processId: Long,
    val dcLogger: IMonitorLogWriter
) : AbstractActor() {

    private val eventStat = EventStat(clusterId, processType, processId, dcLogger)

    companion object {
        fun props(
            clusterId: Long,
            processType: String,
            processId: Long,
            dcLogger: IMonitorLogWriter,
            dispatcher: String,
            mailBox: String
        ): Props {
            return Props.create(HotspotStatsActor::class.java) {
                HotspotStatsActor(clusterId, processType, processId, dcLogger)
            }
                .withDispatcher(dispatcher)
                .withMailbox(mailBox)
        }
    }

    override fun createReceive(): Receive {
        return receiveBuilder()
            .match(EndEvent::class.java) { value ->
                eventStat.end(value)
            }
            .matchEquals("start") {
                eventStat.start()
            }
            .build()
    }
}