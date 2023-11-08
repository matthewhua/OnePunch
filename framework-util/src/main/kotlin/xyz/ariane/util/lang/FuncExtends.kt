package xyz.ariane.util.lang

import java.util.*
import kotlin.collections.HashMap
import kotlin.collections.HashSet

//依赖排序
fun <T> LinkedList<T>.dependSort(dependency: (v: T) -> List<T>?): LinkedList<T> {
    val sorted = LinkedList<T>()
    val visited = HashSet<T>()

    this.forEach {
        visit(it, sorted, visited, dependency)
    }

    return sorted
}

private fun <T> visit(item: T, sorted: LinkedList<T>, visited: HashSet<T>, dependency: (v: T) -> List<T>?) {
    if (!visited.contains(item)) {
        visited.add(item)
        dependency(item)?.forEach {
            visit(it, sorted, visited, dependency)
        }
        sorted.add(item)
    } else {
        if (!sorted.contains(item)) {
            throw RuntimeException("Cyclic dependency found")
        }
    }
}

fun <K, V> Map<K, V>.toHashMap(): HashMap<K, V> {
    val hMap = hashMapOf<K, V>()
    for ((k, v) in this) {
        hMap[k] = v
    }
    return hMap
}

fun <K, V> Map<K, V>.firstOrNull(predicate: (Map.Entry<K, V>) -> Boolean): Map.Entry<K, V>? {
    for (element in this) {
        if (predicate(element)) {
            return element
        }
    }
    return null
}

fun <K, V> Map<K, V>.getFirstOrNull(): Pair<K, V>? {
    for (element in this) {
        return Pair(element.key, element.value)
    }
    return null
}

fun <K> HashMap<K, Int>.plusValue(k: K, v: Int) {
    this[k] = this.getOrDefault(k, 0) + v
}

fun <K> HashMap<K, Long>.plusValue(k: K, v: Long) {
    this[k] = this.getOrDefault(k, 0) + v
}

fun <K> HashMap<K, Int>.minusValue(k: K, v: Int) {
    this[k] = this.getOrDefault(k, 0) - v
}

fun <K> HashMap<K, Long>.minusValue(k: K, v: Long) {
    this[k] = this.getOrDefault(k, 0) - v
}

inline fun <T> Iterable<T>.sumByLong(selector: (T) -> Long): Long {
    var sum: Long = 0
    for (element in this) {
        sum += selector(element)
    }
    return sum
}

fun <K, V> checkMapSame(oldMap: Map<K, V>, newMap: Map<K, V>): Boolean {
    if (oldMap.size != newMap.size) {
        return false
    }
    for ((k, v) in oldMap) {
        val newV = newMap[k]
        if (newV == null) {
            return false
        }
        if (v != newV) {
            return false
        }
    }
    return true
}

fun <K> checkListSame(oldList: List<K>, newList: List<K>): Boolean {
    if (oldList.size != newList.size) {
        return false
    }
    for (v in oldList) {
        val newV = newList.firstOrNull { it == v }
        if (newV == null) {
            return false
        }
    }
    return true
}

fun <R, S, T> List<R>.toMap(
    fetchS: (R) -> S,
    fetchT: (R) -> T,
    enableValueRepeat: Boolean = false,
    enableKeyRepeat: Boolean = false
): Pair<Boolean, Map<S, T>> {
    val resultMap: HashMap<S, T> = hashMapOf()
    if (this.isEmpty()) {
        return Pair(true, resultMap)
    }
    for (r in this) {
        val s = fetchS(r)
        val t = fetchT(r)
        if (resultMap[s] != null && !enableKeyRepeat) {
            return Pair(false, hashMapOf())
        }
        if (resultMap.values.contains(t) && !enableValueRepeat) {
            return Pair(false, hashMapOf())
        }
        resultMap[s] = t
    }
    return Pair(true, resultMap)
}

fun <R, S, T> List<R>.flatToMap(
    fetchS: (R) -> S,
    fetchT: (R) -> T
): Map<S, T> {
    if (this.isEmpty()) {
        return mapOf()
    }

    val resultMap: HashMap<S, T> = hashMapOf()
    for (r in this) {
        val s = fetchS(r)
        val t = fetchT(r)
        resultMap[s] = t
    }
    return resultMap
}

fun <K, V> TreeMap<K, V>.lastValue(): V? {
    val entry = this.lastEntry() ?: return null
    return entry.value
}

fun <R, S> List<R>.toSet(fetchS: (R) -> S, allowRepeat: Boolean = false): Pair<Boolean, Set<S>> {
    val desSet = hashSetOf<S>()
    if (this.isEmpty()) {
        return Pair(true, desSet)
    }
    for (src in this) {
        val s = fetchS(src)
        if (desSet.contains(s) && !allowRepeat) {
            return Pair(true, hashSetOf())
        }
        desSet.add(s)
    }
    return Pair(true, desSet)
}

// list转换
fun <K, T> List<K>.toListBy(transform: (K) -> T): List<T> {
    val result = LinkedList<T>()
    if (this.isEmpty()) {
        return result
    }
    for (i in 0 until this.size) {
        result.addLast(transform(this[i]))
    }
    return result
}

// list转换为PairList
fun <K, T, V> List<K>.toPairListBy(transform: (K) -> Pair<T, V>): List<Pair<T, V>> {
    val list = LinkedList<Pair<T, V>>()
    if (this.isEmpty()) {
        return list
    }
    for (i in 0 until this.size) {
        list.addLast(transform(this[i]))
    }
    return list
}

//数组分割
fun <T> List<T>.split(itemNum: Int): List<List<T>> {
    if (this.isEmpty() || itemNum <= 0) {
        return emptyList()
    }

    val result = LinkedList<List<T>>()
    if (this.size <= itemNum) {
        // 源List元素数量小于等于目标分组数量
        result.add(this)
        return result
    }

    var splitNum = this.size / itemNum
    if (this.size % itemNum != 0) {
        splitNum++
    }

    for (i in 0 until splitNum) {
        val value = if (i < splitNum - 1) {
            this.subList(i * itemNum, (i + 1) * itemNum)
        } else {
            this.subList(i * itemNum, this.size)
        }
        result.add(value)
    }

    return result
}
// list拼接
fun <T> List<T>.appendList(list: List<T>): List<T> {
    val rt = LinkedList<T>()
    for(i in this.indices) {
        rt.addLast(this[i])
    }
    for(i in list.indices) {
        rt.addLast(list[i])
    }
    return rt
}

// list拼接
fun <T> List<T>.appendListWriteable(list: List<T>): LinkedList<T> {
    val rt = LinkedList<T>()
    for (i in this.indices) {
        rt.addLast(this[i])
    }
    for (i in list.indices) {
        rt.addLast(list[i])
    }
    return rt
}
// list拼接
fun <T> List<T>.appendListPlus(list: List<Any>, convert: (Any) -> T): List<T> {
    val rt = LinkedList<T>()
    for (i in this.indices) {
        rt.addLast(this[i])
    }
    for (i in list.indices) {
        rt.addLast(convert(list[i]))
    }
    return rt
}

// list拼接
fun <T> List<T>.appendListPlusWriteable(list: List<Any>, convert: (Any) -> T): LinkedList<T> {
    val rt = LinkedList<T>()
    for (i in this.indices) {
        rt.addLast(this[i])
    }
    for (i in list.indices) {
        rt.addLast(convert(list[i]))
    }
    return rt
}
