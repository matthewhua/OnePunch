package xyz.ariane.util.rank

import xyz.ariane.util.annotation.AllOpen
import xyz.ariane.util.lang.lastValue
import java.util.*
import kotlin.collections.HashMap

/**
 * 排行信息类
 *
 * KEY：值对应的唯一键
 * SORT_BASIS：根据值获取的排序依据
 * VALUE：值
 * SORT_BASIS_FOR_SAME_SCORE: 同名次排行依据
 * comparator：比较器
 * maxNum:排行榜最大存数 -1表示无限制
 *
 * 注意：由于rankMap中存在成员是函数，不能存库，一旦存库反序列化将会失败!
 */
@AllOpen
open class RankMap<KEY : Comparable<KEY>, SORT_BASIS : Comparable<SORT_BASIS>, VALUE, SORT_BASIS_FOR_SAME_SCORE>(
    private val fetchKey: (v: VALUE) -> KEY, // 从值中获取Key
    private val fetchSortAccording: (v: VALUE) -> SORT_BASIS, // 从值中获取排序主值
    private val fetchSameScoreSortAccording: (v: VALUE) -> SORT_BASIS_FOR_SAME_SCORE, // 从值中获取排序次值
    comparator: Comparator<SORT_BASIS>, // 排序方式
    private val sameScoreComparator: Comparator<SORT_BASIS_FOR_SAME_SCORE>,
    private val minValue: SORT_BASIS, // 高于这个值才能进入排行榜
    private var maxNum: Int = -1 // 设置上榜最大条数 -1表示无条件
) {
    private val rankMap =
        TreeMap<SORT_BASIS, TreeMap<SORT_BASIS_FOR_SAME_SCORE, TreeMap<KEY, VALUE>>>(comparator) // 排序的map

    // V的两个排序值的旧值缓存！
    // 因为更新排行时，V的两个值可能已经变化了，所以需要下面的缓存来更新排行
    private val indexMap = HashMap<KEY, SORT_BASIS>() // 保存对象的主排序值
    private val indexMap1 = HashMap<KEY, SORT_BASIS_FOR_SAME_SCORE>() // 保存对象的次排序值

    /**
     * 插入/更新值
     * 返回尝试插入/更新的元素最终有没有在排行里
     */
    fun updateValue(v: VALUE, removeCb: (VALUE) -> Unit = {}): Boolean {
        var isIn = false
        //先移除旧记录
        removeValue(v)

        // 重新插入记录
        val t = fetchSortAccording(v)
        if (t > minValue) {
            isIn = true
            val k = fetchKey(v)
            val u = fetchSameScoreSortAccording(v)
            indexMap[k] = t
            indexMap1[k] = u

            // 存入排序容器
            val sameRankMap = rankMap.getOrPut(t) { TreeMap(sameScoreComparator) }
            sameRankMap.getOrPut(u) { TreeMap() }[k] = v

            if (maxNum != -1 && queryAllJoinNum() > maxNum) {
                // 如果排行榜有存数上限需求 并且当前已经超出 就删除尾部元素
                val delValue = rankMap.lastValue()?.lastValue()?.lastValue()
                if (delValue == null) {
                    return isIn
                }
                if (removeValue(delValue) != null) {
                    removeCb(delValue)
                }

                if (fetchKey(delValue) == fetchKey(v)) {
                    // 插入之后又被移出去了
                    isIn = false
                }
            }
        }

        return isIn
    }

    /**
     * 移除值
     */
    fun removeValue(v: VALUE): VALUE? {
        val k = fetchKey(v)
        return removeByKey(k)
    }

    fun removeByKey(k: KEY): VALUE? {
        val oldT = indexMap[k]
        if (oldT == null) {
            return null
        }
        val oldU = indexMap1[k]
        if (oldU == null) {
            return null
        }

        // 移除旧记录
        indexMap.remove(k)
        indexMap1.remove(k)

        val treeMap1 = rankMap[oldT] ?: return null
        val treeMap2 = treeMap1[oldU] ?: return null
        val v = treeMap2.remove(k)
        if (treeMap2.isEmpty()) {
            treeMap1.remove(oldU)
        }
        if (treeMap1.isEmpty()) {
            rankMap.remove(oldT)
        }
        return v
    }

    fun clear() {
        rankMap.clear()
        indexMap.clear()
        indexMap1.clear()
    }

    /**
     * 修改最大条目
     */
    fun changeMaxNum(num: Int) {
        this.maxNum = num
    }

    /**
     * 根据键查找当前值
     */
    fun findByKey(k: KEY): VALUE? {
        val t = indexMap[k]
        if (t == null) {
            return null
        }
        val u = indexMap1[k]
        if (u == null) {
            return null
        }

        val treeMap = this.rankMap[t]
        if (treeMap == null) {
            return null
        }
        return treeMap[u]?.get(k)
    }

    /**
     * 查询前N名
     */
    fun queryValue(num: Int): LinkedList<VALUE> {
        val valueList = LinkedList<VALUE>()
        val iterator1 = rankMap.iterator()
        while (iterator1.hasNext() && valueList.count() < num) {
            val valueEntry1 = iterator1.next()
            val iterator2 = valueEntry1.value.iterator()
            while (iterator2.hasNext() && valueList.count() < num) {
                val valueEntry2 = iterator2.next()
                val iterator3 = valueEntry2.value.iterator()
                while (iterator3.hasNext() && valueList.count() < num) {
                    valueList.add(iterator3.next().value)
                }
            }
        }
        return valueList
    }

    /**
     * 获取总参与排行人数
     */
    fun queryAllJoinNum(): Int {
        var joinNum = 0
        val iterator1 = rankMap.iterator()
        while (iterator1.hasNext()) {
            val valueEntry1 = iterator1.next()
            val iterator2 = valueEntry1.value.iterator()
            while (iterator2.hasNext()) {
                val valueEntry2 = iterator2.next()
                joinNum += valueEntry2.value.count()
            }
        }

        return joinNum
    }

    /**
     * 查询第N名 - 第M名
     */
    fun queryPartValue(startNum: Int, endNum: Int): LinkedList<VALUE> {
        var nowValue = 0
        val valueList = LinkedList<VALUE>()
        val iterator1 = rankMap.iterator()
        while (iterator1.hasNext()) {
            val valueEntry1 = iterator1.next()
            val iterator2 = valueEntry1.value.iterator()
            while (iterator2.hasNext()) {
                val valueEntry2 = iterator2.next()
                val iterator3 = valueEntry2.value.iterator()
                while (iterator3.hasNext()) {
                    nowValue += 1
                    val aValue = iterator3.next()
                    if (nowValue in startNum..endNum) {
                        valueList.add(aValue.value)
                    }
                }
            }
        }

        return valueList
    }

    /**
     * 查询指定排名的数据
     */
    fun querySingleValue(num: Int): VALUE? {
        var nowValue = 0
        val iterator1 = rankMap.iterator()
        while (iterator1.hasNext()) {
            val valueEntry1 = iterator1.next()
            val iterator2 = valueEntry1.value.iterator()
            while (iterator2.hasNext()) {
                val valueEntry2 = iterator2.next()
                val iterator3 = valueEntry2.value.iterator()
                while (iterator3.hasNext()) {
                    nowValue += 1
                    val aValue = iterator3.next()
                    if (nowValue == num) {
                        return aValue.value
                    }
                }
            }
        }
        return null
    }


    /**
     * 查询排名
     */
    fun queryRank(k: KEY): Int {
        val t = indexMap[k]
        if (t == null) {
            return 0
        }
        val u = indexMap1[k]
        if (u == null) {
            return 0
        }

        var rank = 0
        val iterator1 = rankMap.iterator()
        while (iterator1.hasNext()) {
            val valueEntry1 = iterator1.next()
            val iterator2 = valueEntry1.value.iterator()
            while (iterator2.hasNext()) {
                val valueEntry2 = iterator2.next()
                if (t != valueEntry1.key || u != valueEntry2.key) {
                    rank += valueEntry2.value.size
                    continue
                }

                val iterator3 = valueEntry2.value.iterator()
                while (iterator3.hasNext()) {
                    rank++
                    val valueEntry3 = iterator3.next()
                    if (k == valueEntry3.key) {
                        return rank
                    }
                }
            }
        }
        return rank
    }

    /**
     * 查询可同分的排行
     */
    fun queryCanSameRank(k: KEY): Int {
        val t = indexMap[k]
        if (t == null) {
            return 0
        }
        val u = indexMap1[k]
        if (u == null) {
            return 0
        }

        var rank = 1
        val iterator1 = rankMap.iterator()
        while (iterator1.hasNext()) {
            val valueEntry1 = iterator1.next()

            if (valueEntry1.key != t) {
                val iterator2 = valueEntry1.value.iterator()
                while (iterator2.hasNext()) {
                    val valueEntry2 = iterator2.next()
                    rank += valueEntry2.value.size
                }
                continue
            }

            return rank
        }
        return 0
    }

    fun foreachValue(cb: (rank: Int, v: VALUE) -> Unit) {
        var rank = 0
        val iterator1 = rankMap.iterator()
        while (iterator1.hasNext()) {
            val valueEntry1 = iterator1.next()
            val iterator2 = valueEntry1.value.iterator()
            while (iterator2.hasNext()) {
                val valueEntry2 = iterator2.next()
                val iterator3 = valueEntry2.value.iterator()
                while (iterator3.hasNext()) {
                    rank++
                    val valueEntry3 = iterator3.next()
                    cb(rank, valueEntry3.value)
                }
            }
        }
    }

    /**
     * 通过V传出外层的排行依据
     */
    fun findTByV(v: VALUE): SORT_BASIS {
        return fetchSortAccording(v)
    }

    /**
     * 通过V传出内层的排行依据
     */
    fun find2ndTByV(v: VALUE): SORT_BASIS_FOR_SAME_SCORE {
        return fetchSameScoreSortAccording(v)
    }

    /**
     * 返回排行榜里的最后一个entity
     */
    fun findLastEntity(): SORT_BASIS? {
        val v = rankMap.lastValue()?.lastValue()?.lastValue()
        if (v != null) {
            return findTByV(v)
        } else {
            return null
        }
    }

    fun findMinScore(): SORT_BASIS {
        return minValue
    }

    fun nowMapCount(): Int {
        return rankMap.size
    }
}
