package xyz.ariane.util.memodbupgrade

import xyz.ariane.util.memodb.UpgradeExpandableEntityWrapper

abstract class AbstractVersionDbDataUpgrade(val targetAav: Int) {

    // 升级handler表
    var specialDbDataUpgrade: MutableMap<String, IDbDataUpgradeHandler> = mutableMapOf()

    fun upgrade(data: UpgradeExpandableEntityWrapper<*>) {
        if (data.v.aav >= targetAav) {
            return
        }

        val handler = specialDbDataUpgrade[data::class.java.simpleName]
        if (handler == null) {
            return
        }

        handler.upgrade(data)

        data.v.aav = targetAav
    }

}