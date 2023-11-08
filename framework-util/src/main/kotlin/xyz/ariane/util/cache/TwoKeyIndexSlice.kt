package xyz.ariane.util.cache

import com.fasterxml.jackson.annotation.JsonIgnore
import java.util.*
import kotlin.collections.HashMap

/**
 * 注意：不建议用于需要参与序列化并且存数据库的字段!
 */
data class TwoKeyIndexSlice<K, T, V> protected constructor(
    var index: HashMap<K, java.util.HashMap<T, LinkedList<V>>> = hashMapOf()
) {
    @JsonIgnore
    @Transient
    private lateinit var fetch1Key: (v: V) -> K

    @JsonIgnore
    @Transient
    private lateinit var fetch2Key: (v: V) -> T

    @JsonIgnore
    @Transient
    private lateinit var equal: (itEle: V, targetVal: V) -> Boolean

    @JsonIgnore
    @Transient
    private var ignoreIndexCheck: ((k: K, t: T) -> Boolean)? = null

    constructor(fetch1: (v: V) -> K, fetch2: (v: V) -> T, equalFunc: (itEle: V, targetVal: V) -> Boolean) : this() {
        this.fetch1Key = fetch1
        this.fetch2Key = fetch2
        this.equal = equalFunc
    }

    constructor (
        fetch1: (v: V) -> K,
        fetch2: (v: V) -> T,
        equalFunc: (itEle: V, targetVal: V) -> Boolean,
        check: (k: K, t: T) -> Boolean
    ) : this(fetch1, fetch2, equalFunc) {
        this.ignoreIndexCheck = check
    }

    fun indexLen(k: K, t: T): Int {
        val valMap = this.index[k]
        if (valMap === null) {
            return 0
        }
        val valList = valMap[t]
        if (valList === null) {
            return 0
        }
        return valList.size
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
        val valueList = valueMap.getOrPut(key2) { LinkedList() }
        valueList.add(targetValue)
    }

    fun deleteByKey(targetValue: V) {
        val key1 = this.fetch1Key(targetValue)
        val valMap = this.index[key1]
        if (valMap === null) {
            return
        }
        val key2 = this.fetch2Key(targetValue)
        val valList = valMap[key2]
        if (valList === null) {
            return
        }
        valList.removeIf { value: V -> this.equal(value, targetValue) }
        if (valList.size == 0) {
            valMap.remove(key2)
        }
        if (valMap.isEmpty()) {
            this.index.remove(key1)
        }
    }

    fun deleteByOneKey(key1: K) {
        this.index.remove(key1)
    }

    fun findByKey(key1: K, key2: T, filter: (value: V) -> Boolean) {
        val valMap = this.index[key1]
        if (valMap === null) {
            return
        }
        val valList = valMap[key2]
        if (valList === null) {
            return
        }
        for (value in valList) {
            if (!filter(value)) {
                break
            }
        }
    }

    fun findByOneKeyFilter(key1: K, filter: (value: V) -> Boolean) {
        val valMap = this.index[key1]
        if (valMap === null) {
            return
        }
        for ((_, valList) in valMap) {
            for (value in valList) {
                if (!filter(value)) {
                    break
                }
            }
        }
    }

    fun findByOneKey(key1: K): List<V> {
        val valList = LinkedList<V>()
        val valMap = this.index[key1]
        valMap?.let {
            for ((_, values) in it) {
                valList.addAll(values)
            }
        }
        return valList
    }

    fun findByTwoKey(key2: T): List<V> {
        val valList = LinkedList<V>()
        for ((_, valMap) in this.index) {
            val values = valMap[key2]
            if (values != null) {
                valList.addAll(values)
            }
        }
        return valList
    }

    fun map(filter: (v: V) -> Boolean) {
        for ((_, valMap) in this.index) {
            for ((_, valList) in valMap) {
                for (value in valList) {
                    if (!filter(value)) {
                        return
                    }
                }
            }
        }
    }

    fun sizeByKey(key1: K, key2: T): Int {
        val valMap = this.index[key1]?.get(key2)
        if (valMap === null) {
            return 0
        }
        return valMap.size
    }
}
