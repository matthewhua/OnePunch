package xyz.ariane.util.memodb

import com.fasterxml.jackson.module.kotlin.jacksonTypeRef
import xyz.ariane.util.json.mapperUsedInGame
import xyz.ariane.util.json.toObj
import xyz.ariane.util.memodbupgrade.AbstractVersionDbDataUpgrade

abstract class UpgradeExpandableEntityWrapper<T : IWrapEntity> : EntityWrapper<T> {

    /** 子类需要实现此属性并加入[hashCode]和[equals]中 */
    abstract var v: T

    /**
     * 当前包裹类转数据entity对象
     */
    final override fun toEntity(): T = copyEntity().also(this::collapse)

    /**
     * 克隆一个entity对象，用于生成存库操作
     * **不能直接返回[v]，因为存库操作互在其他线程读取数据，线程不安全**
     */
    protected abstract fun copyEntity(): T

    /** 把展开的内存数据折叠回[e]中，用于生成存库操作 */
    protected abstract fun collapse(e: T)

    /**
     * 将entity对象包裹进一个包裹类中，方便后续逻辑操作。
     */
    final override fun wrap(upgradeHandlers: List<AbstractVersionDbDataUpgrade>, entity: T) {
        this.v = entity

        expand(upgradeHandlers, entity)
    }

    /**
     * 展开[e]中的折叠数据到内存数据结构中，
     * **所有除[v]外的自定义数据结构都应该在此函数内初始化**
     */
    private fun expand(upgradeHandlers: List<AbstractVersionDbDataUpgrade>, e: T) {
        // 展开数据
        expandData(v)

        // 新的方式升级
        for (handler in upgradeHandlers) {
            handler.upgrade(this)
        }
    }

    abstract fun expandData(tempEntity: T)

    inline fun <reified K : Any> supplyExpandDirect(
        dataBytes: ByteArray,
        wrapData: (K) -> Unit
    ) {
        if (dataBytes.isEmpty()) {
            return
        }

        val obj = toObj<K>(dataBytes)

        wrapData(obj)
    }

    inline fun <reified K : Any> supplyExpandDirect(
        dataBytes: ByteArray,
        dataLength: Int,
        wrapData: (K) -> Unit
    ) {
        if (dataBytes.isEmpty()) {
            return
        }

        val obj = mapperUsedInGame.readValue<K>(dataBytes, 0, dataLength, jacksonTypeRef())

        wrapData(obj)
    }

    inline fun <reified K : Any> supplyExpand(
        dataStr: String,
        wrapData: (K) -> Unit
    ) {
        if (dataStr.isEmpty()) {
            return
        }

        val obj = toObj<K>(dataStr)

        wrapData(obj)
    }
}