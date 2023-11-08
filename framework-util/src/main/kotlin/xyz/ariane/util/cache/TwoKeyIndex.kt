package xyz.ariane.util.cache

import com.fasterxml.jackson.annotation.JsonIgnore
import java.util.*

/**
 * 注意：不建议用于需要参与序列化并且存数据库的字段!
 */
data class TwoKeyIndex<K, T, V> protected constructor(
    var index: HashMap<K, HashMap<T, V>> = hashMapOf()
) {
    @JsonIgnore
    @Transient
    lateinit var fetch1Key: (v: V) -> K

    @JsonIgnore
    @Transient
    lateinit var fetch2Key: (v: V) -> T

    @JsonIgnore
    @Transient
    private var ignoreIndexCheck: ((k: K, t: T) -> Boolean)? = null

    constructor(fetch1: (v: V) -> K, fetch2: (v: V) -> T) : this() {
        this.fetch1Key = fetch1
        this.fetch2Key = fetch2
    }

    constructor (fetch1: (v: V) -> K, fetch2: (v: V) -> T, check: (k: K, t: T) -> Boolean) : this(fetch1, fetch2) {
        this.ignoreIndexCheck = check
    }

    fun fetchAllValues(): List<V> {
        val rt = LinkedList<V>()
        for ((_, vm) in index) {
            rt.addAll(vm.values)
        }
        return rt
    }

    fun indexLen(): Int {
        var num = 0
        for ((_, map) in this.index) {
            num += map.size
        }
        return num
    }

    fun updateByKey(newKey1: K, newKey2: T, targetValue: V, updateKeyCb: () -> Unit) {
        val oldKey1 = this.fetch1Key(targetValue)
        val oldKey2 = this.fetch2Key(targetValue)
        if (newKey1 == oldKey1 && newKey2 == oldKey2) {
            return
        }

        this.deleteByKey(targetValue)

        val check = this.ignoreIndexCheck
        if (check == null || !check(newKey1, newKey2)) {
            this.move2NewIndex(newKey1, newKey2, targetValue)
        }

        updateKeyCb()
    }

    fun insertByKey(targetValue: V) {
        val key1 = this.fetch1Key(targetValue)
        val key2 = this.fetch2Key(targetValue)
        val check = this.ignoreIndexCheck
        if (check == null || !check(key1, key2)) {
            this.move2NewIndex(key1, key2, targetValue)
        }
    }

    private fun move2NewIndex(key1: K, key2: T, targetValue: V) {
        val valueMap = this.index.getOrPut(key1) { hashMapOf() }
        valueMap[key2] = targetValue
    }

    fun deleteByKey(targetValue: V) {
        val key1 = this.fetch1Key(targetValue)
        val valMap = this.index[key1]
        if (valMap === null) {
            return
        }
        val key2 = this.fetch2Key(targetValue)
        valMap.remove(key2)
        if (valMap.isEmpty()) {
            this.index.remove(key1)
        }
    }

    fun deleteById(key1: K, key2: T): V? {
        val valMap = this.index[key1]
        if (valMap === null) {
            return null
        }
        val v = valMap.remove(key2)
        if (valMap.isEmpty()) {
            this.index.remove(key1)
        }
        return v
    }

    fun deleteByOneKey(key1: K) {
        this.index.remove(key1)
    }

    fun findByKey(key1: K, key2: T): V? {
        val valMap = this.index[key1]
        if (valMap === null) {
            return null
        }
        return valMap[key2]
    }

    fun findByOneKeyFilter(key1: K, filter: (value: V) -> Boolean) {
        val valMap = this.index[key1]
        if (valMap === null) {
            return
        }
        for ((_, value) in valMap) {
            if (!filter(value)) {
                break
            }
        }
    }

    fun sizeByOneKey(key1: K): Int {
        val valMap = this.index[key1]
        if (valMap === null) {
            return 0
        }
        return valMap.size
    }

    fun findByOneKey(key1: K): List<V> {
        val valList = LinkedList<V>()
        val valMap = this.index[key1]
        valMap?.let {
            for ((_, value) in it) {
                valList.add(value)
            }
        }
        return valList
    }

    fun findByTwoKey(key2: T): List<V> {
        val valList = LinkedList<V>()
        for ((_, valMap) in this.index) {
            val value = valMap[key2]
            if (value != null) {
                valList.add(value)
            }
        }
        return valList
    }

    fun getOrPut(key1: K, key2: T, fetchV: () -> V): V {
        var v = findByKey(key1, key2)
        if (v != null) {
            return v
        }
        v = fetchV()
        insertByKey(v)
        return v
    }

    fun map(filter: (v: V) -> Boolean) {
        for ((_, valMap) in this.index) {
            for ((_, value) in valMap) {
                if (!filter(value)) {
                    break
                }
            }
        }
    }

    fun keys(): LinkedList<K> {
        val keys = LinkedList<K>()
        for ((k, _) in this.index) {
            keys.add(k)
        }
        return keys
    }

    /**
     * 获取key1对应的子Map，只读！
     */
    fun subMaps(key1: K): Map<T, V>? {
        return index[key1]
    }

    fun clear() {
        this.index.clear()
    }
}
