package xyz.ariane.util.lang

import java.lang.Math.addExact
import java.util.*

fun <K> HashMap<K, Int>.mergePlus(another: Map<K, Int>) {
    for ((k, v) in another) {
        merge(k, v, ::addExact)
    }
}

fun <K> HashMap<K, Int>.mergePlus(pair: Pair<K, Int>) {
    val (k, v) = pair
    merge(k, v, ::addExact)
}

fun <K> HashMap<K, Int>.mergePlus(entry: Map.Entry<K, Int>) {
    val (k, v) = entry
    merge(k, v, ::addExact)
}

inline fun <T> Iterable<T>.checkedSumBy(toInt: (T) -> Int): Int = fold(0) { s, e -> addExact(s, toInt(e)) }

inline fun <T> Iterable<T>.checkedSumByLong(toLong: (T) -> Long): Long = fold(0L) { s, e -> addExact(s, toLong(e)) }

inline fun <T> Array<out T>.checkedSumBy(toInt: (T) -> Int): Int = fold(0) { s, e -> addExact(s, toInt(e)) }

inline fun <T> Array<out T>.checkedSumByLong(toLong: (T) -> Long): Long = fold(0L) { s, e -> addExact(s, toLong(e)) }

fun Iterable<Int>.checkedSum(): Int = fold(0, ::addExact)

fun Iterable<Long>.checkedSum(): Long = fold<Long, Long>(0L, ::addExact)

fun IntArray.checkedSum(): Int = fold(0, ::addExact)

fun LongArray.checkedSum(): Long = fold<Long>(0L, ::addExact)


/**
 * 按各个元素的权重比分配数值，余数给定一个分配函数 remainAllocate
 */
fun <T> Collection<T>.weightAllot(
    totalValue: Long,
    getWeight: (T) -> Long,
    remainAllocate: (T, Long, Long) -> Long
): Map<T, Long> {
    val valueMap = hashMapOf<T, Long>()
    var totalWeight = 0L
    for (role in this) {
        totalWeight += getWeight(role)
    }
    // 按权重分配
    var allocatedValue = 0L
    for (role in this) {
        val value = (totalValue * getWeight(role) / totalWeight)
        valueMap[role] = value
        allocatedValue += value
    }
    // 余数分配
    if (totalValue > allocatedValue) {
        for (role in this) {
            val left = totalValue - allocatedValue
            if (left <= 0) {
                break
            }
            val value = remainAllocate(role, valueMap.getOrDefault(role, 0), left)
            valueMap.plusValue(role, value)
            allocatedValue += value
        }
    }
    return valueMap
}
