package xyz.ariane.util.memodb

import akka.actor.ActorRef
import xyz.ariane.util.lang.profile

/**
 * 同步数据库操作
 */
class SyncOp(perfMonitor: ActorRef, tracker: ModificationTracker, val op: ModificationTracker.Op) {

    val entity: IEntity = profile(false, perfMonitor, "to_${tracker.entityWrapper.javaClass.name}") {
        // toEntity()方法一般会复制一份entity出来！
        tracker.entityWrapper.toEntity()
    }

    val key: EntityWrapperKey = tracker.key

    override fun toString(): String = "SyncOp@${System.identityHashCode(this)}($op, $key)"
}