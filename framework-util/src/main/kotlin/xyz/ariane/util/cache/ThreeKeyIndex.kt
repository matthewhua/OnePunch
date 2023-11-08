package xyz.ariane.util.cache

import com.fasterxml.jackson.annotation.JsonIgnore
import java.util.*

/**
 * 注意：不建议用于需要参与序列化并且存数据库的字段!
 */
data class ThreeKeyIndex<K, T, U, V> protected constructor(
    var index: HashMap<K, HashMap<T, HashMap<U, V>>> = hashMapOf()
) {
    @JsonIgnore
    @Transient
    private lateinit var fetch1Key: (v: V) -> K

    @JsonIgnore
    @Transient
    private lateinit var fetch2Key: (v: V) -> T

    @JsonIgnore
    @Transient
    private lateinit var fetch3Key: (v: V) -> U

    @JsonIgnore
    @Transient
    private var ignoreIndexCheck: ((k: K, t: T, u: U) -> Boolean)? = null

    constructor(fetch1: (v: V) -> K, fetch2: (v: V) -> T, fetch3: (v: V) -> U) : this() {
        this.fetch1Key = fetch1
        this.fetch2Key = fetch2
        this.fetch3Key = fetch3
    }

    constructor (
        fetch1: (v: V) -> K,
        fetch2: (v: V) -> T,
        fetch3: (v: V) -> U,
        check: (k: K, t: T, u: U) -> Boolean
    ) : this(fetch1, fetch2, fetch3) {
        this.ignoreIndexCheck = check
    }

    fun indexLen(): Int {
        var num = 0
        for ((_, valMap1) in this.index) {
            for ((_, valMap2) in valMap1) {
                num += valMap2.size
            }
        }
        return num
    }

    fun updateByKey(newKey1: K, newKey2: T, newKey3: U, targetValue: V, updateKeyCb: () -> Unit) {
        val oldKey1 = this.fetch1Key(targetValue)
        val oldKey2 = this.fetch2Key(targetValue)
        val oldKey3 = this.fetch3Key(targetValue)
        if (newKey1 == oldKey1 && newKey2 == oldKey2 && newKey3 == oldKey3) {
            return
        }

        this.deleteByKey(targetValue)

        val check = this.ignoreIndexCheck
        if (check == null || !check(newKey1, newKey2, newKey3)) {
            this.move2NewIndex(newKey1, newKey2, newKey3, targetValue)
        }

        updateKeyCb()
    }

    fun insertByKey(targetValue: V) {
        val key1 = this.fetch1Key(targetValue)
        val key2 = this.fetch2Key(targetValue)
        val key3 = this.fetch3Key(targetValue)
        val check = this.ignoreIndexCheck
        if (check == null || !check(key1, key2, key3)) {
            this.move2NewIndex(key1, key2, key3, targetValue)
        }
    }

    private fun move2NewIndex(key1: K, key2: T, key3: U, targetValue: V) {
        val valueMap1 = this.index.getOrPut(key1) { hashMapOf() }
        val valueMap2 = valueMap1.getOrPut(key2) { hashMapOf() }
        valueMap2[key3] = targetValue
    }

    fun deleteByKey(targetValue: V) {
        val key1 = this.fetch1Key(targetValue)
        val valMap1 = this.index[key1]
        if (valMap1 === null) {
            return
        }
        val key2 = this.fetch2Key(targetValue)
        val valMap2 = valMap1[key2]
        if (valMap2 === null) {
            return
        }
        val key3 = this.fetch3Key(targetValue)
        valMap2.remove(key3)
        if (valMap2.isEmpty()) {
            valMap1.remove(key2)
        }
        if (valMap1.isEmpty()) {
            this.index.remove(key1)
        }
    }

    fun deleteByOneKey(key1: K) {
        this.index.remove(key1)
    }

    fun deleteWithKey(key1: K, key2: T, key3: U) {
        val valMap1 = this.index[key1]
        if (valMap1 === null) {
            return
        }
        val valMap2 = valMap1[key2]
        if (valMap2 === null) {
            return
        }
        valMap2.remove(key3)
        if (valMap2.isEmpty()) {
            valMap1.remove(key2)
        }
        if (valMap1.isEmpty()) {
            this.index.remove(key1)
        }
    }

    fun findByKey(key1: K, key2: T, key3: U): V? {
        val valMap1 = this.index[key1]
        if (valMap1 === null) {
            return null
        }
        val valMap2 = valMap1[key2]
        if (valMap2 === null) {
            return null
        }
        return valMap2[key3]
    }

    fun findByTwoKey(key1: K, key2: T): HashMap<U, V>? {
        val valMap1 = this.index[key1]
        if (valMap1 === null) {
            return null
        }
        return valMap1[key2]
    }

    fun map(filter: (v: V) -> Boolean) {
        for ((_, valMap1) in this.index) {
            for ((_, valMap2) in valMap1) {
                for ((_, value) in valMap2) {
                    if (!filter(value)) {
                        break
                    }
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

    fun clear() {
        this.index.clear()
    }

    fun clear(idx: K) {
        val hashMap = this.index[idx]
        if (hashMap != null) {
            hashMap.clear()
        }
    }
}
