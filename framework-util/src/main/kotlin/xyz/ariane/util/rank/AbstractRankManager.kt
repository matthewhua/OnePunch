package xyz.ariane.util.rank

import com.google.common.collect.Lists
import org.slf4j.Logger
import xyz.ariane.util.lang.InitializeRequired
import xyz.ariane.util.dclog.lzDebug
import java.lang.reflect.ParameterizedType
import java.util.*
import kotlin.reflect.KMutableProperty1

/** 排行榜一个排位位置 */
abstract class RankPlaceHolder<RT : Enum<RT>, ST : Enum<ST>>(val rankType: RT, val subType: ST, val rank: Int) {
    /** 此位置的排位数据 */
    abstract var rankData: Any
    /** 附加数据，如英雄排行榜上附加的英雄详细信息 */
    @Transient
    open var attachment: Any? = null

    /** 此位置当前排位数据的唯一id */
    abstract fun getDataIdentifier(): Any

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other?.javaClass != javaClass) return false

        other as RankPlaceHolder<*, *>

        if (rankType != other.rankType) return false
        if (subType != other.subType) return false
        if (rank != other.rank) return false
        if (rankData != other.rankData) return false

        return true
    }

    override fun hashCode(): Int {
        var result = rankType.hashCode()
        result = 31 * result + subType.hashCode()
        result = 31 * result + rank
        result = 31 * result + rankData.hashCode()
        return result
    }

}

/**
 * 用于实现按大类和小类（排序列）分类的排行榜
 *
 * @param maxRank 最大排行数
 *
 * @param RT 排行榜大类
 * @param ST 排行榜小类，既一个排行列
 * @param H [RankPlaceHolder]
 * @param D 初始化数据类
 */
abstract class AbstractRankManager<RT : Enum<RT>, ST : Enum<ST>, H : RankPlaceHolder<RT, ST>, in D>(val maxRank: Int) :
    InitializeRequired<D>() {

    private val genericSuperClass: ParameterizedType get() = javaClass.genericSuperclass as ParameterizedType
    @Suppress("UNCHECKED_CAST")
    private val rankTypeEnumClass: Class<RT> = genericSuperClass.actualTypeArguments[0] as Class<RT>
    @Suppress("UNCHECKED_CAST")
    private val subTypeEnumClass: Class<ST> = genericSuperClass.actualTypeArguments[1] as Class<ST>

    private val createRankSeq: () -> ArrayList<H> = { Lists.newArrayListWithCapacity(maxRank) }

    private val createSubTypeMap: () -> EnumMap<ST, ArrayList<H>> = { EnumMap<ST, ArrayList<H>>(subTypeEnumClass) }

    protected abstract val logger: Logger

    private val rankMap: EnumMap<RT, EnumMap<ST, ArrayList<H>>> = EnumMap(rankTypeEnumClass)

    /**
     * 根据大类和小类获取排行榜
     */
    fun rankSeqOf(rankType: RT, subType: ST): List<H> = mutableSubTypeSeqOf(rankType, subType)

    /**
     * 按大类遍历所有子类的排行榜
     */
    fun forEach(rankType: RT, consume: (ST, List<H>) -> Unit) {
        mutableRankTypeMapOf(rankType).forEach { (subType, seq) ->
            consume(subType, seq)
        }
    }

    /** 创建一个新的[RankPlaceHolder]的默认实现 */
    protected abstract fun defaultNewPlaceHolder(rankType: RT, subType: ST, id: Any, rank: Int): H

    /** [RankPlaceHolder]数据存库 */
    protected abstract fun save(holder: H)

    /**
     * 对排行榜数据重新排序
     * @return 有序的排行榜数据
     */
    protected abstract fun sortData(rankSeq: List<H>): List<Any>

    /**
     * 比较两个相同小类的排行榜数据
     * @return true表示本数据比[other]排位更靠前
     */
    abstract fun H.isBetterThan(other: H): Boolean

    /**
     * 是否更新其他排行榜同一数值
     */
    open fun isUpdateOtherSubTypeRank(): Boolean = true

    private fun List<RankPlaceHolder<RT, ST>>.isFull(): Boolean = size >= maxRank

    protected fun mutableSubTypeSeqOf(rankType: RT, subType: ST): ArrayList<H> =
        mutableRankTypeMapOf(rankType).getOrPut(subType, createRankSeq)

    private fun mutableRankTypeMapOf(rankType: RT): MutableMap<ST, ArrayList<H>> =
        rankMap.getOrPut(rankType, createSubTypeMap)

    private fun <V> setValue(holder: H, newValue: V, property: KMutableProperty1<out Any, V>, attachment: Any?) {
        @Suppress("UNCHECKED_CAST")
        (property as KMutableProperty1<Any, V>).set(holder.rankData, newValue)
        holder.attachment = attachment
    }

    protected fun <V> merge(
        id: Any, rt: RT, st: ST,
        newValue: V, property: KMutableProperty1<out Any, V>,
        attachment: Any? = null,
        /** 自定义特殊的创建函数 */
        newPlaceHolder: ((RT, ST, Any, Int) -> H)? = null
    ) {

        // 更新其他排行榜同一数值
        if (isUpdateOtherSubTypeRank()) {
            mutableRankTypeMapOf(rt).forEach { (curSubType, rankSeq1) ->
                if (curSubType != st) {
                    rankSeq1.find { it.getDataIdentifier() == id }
                        ?.let { holder -> setValue(holder, newValue, property, attachment) }
                }
            }
        }
        val rankSeq = mutableSubTypeSeqOf(rt, st)
        val existHolder: H? = rankSeq.find { it.getDataIdentifier() == id }
        if (existHolder != null) {
            setValue(existHolder, newValue, property, attachment)
        } else {
            val lastRankNum = rankSeq.size + 1
            val newHolder =
                newPlaceHolder?.invoke(rt, st, id, lastRankNum) ?: defaultNewPlaceHolder(rt, st, id, lastRankNum)
            setValue(newHolder, newValue, property, attachment)

            val full = rankSeq.isFull()
            if (full && !newHolder.isBetterThan(rankSeq.last())) {
                return // 不能上榜
            }
            rankSeq.add(newHolder) // 可能排行榜未满，也可能可以替换下其他已上榜的人，后面会踢掉被替换下的人
            if (!full) {
                save(newHolder) // 新增排行榜位置
            }
        }

        if (rankSeq.isNotEmpty()) {
            sortData(rankSeq).forEachIndexed { i, data ->
                injectSortedRankData(rankSeq[i], data)
            }
            while (rankSeq.size > maxRank) {
                val removed = rankSeq.removeAt(rankSeq.lastIndex)
                logger.lzDebug { "$removed removed" }
            }
        }
    }

    protected open fun injectSortedRankData(holder: H, data: Any) {
        holder.rankData = data
    }

    /** 调用者需提前delete所有entity */
    protected fun dangerClearAllRankInMemory() {
        rankMap.clear()
    }

    protected fun dangerClearTargetRankInMemory(rankType: RT, smallRankType: ST) {
        rankMap[rankType]?.get(smallRankType)?.clear()
    }
}