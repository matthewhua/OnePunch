package xyz.ariane.util.cache

import com.fasterxml.jackson.annotation.JsonIgnore

/**
 * 注意：不建议用于需要参与序列化并且存数据库的字段!
 */
data class OneKeyIndex<K, V> protected constructor(
    val index: HashMap<K, V> = hashMapOf()
) {
    @JsonIgnore
    @Transient
    lateinit var fetch1Key: (v: V) -> K  // 输入值，返回键的函数

    @JsonIgnore
    @Transient
    private var ignoreIndexCheck: ((v: V) -> Boolean)? = null // 输入键，返回bool的函数

    constructor(fetch: (v: V) -> K) : this() {
        this.fetch1Key = fetch
    }

    constructor (fetch: (v: V) -> K, check: (v: V) -> Boolean) : this(fetch) {
        this.ignoreIndexCheck = check
    }

    fun indexLen(): Int {
        return this.index.size
    }

    fun updateByKey(newKey: K, targetValue: V, updateKeyCb: () -> Unit) {
        val oldKey = this.fetch1Key(targetValue)
        if (newKey == oldKey) {
            return
        }

        this.deleteByKey(targetValue)

        val check = this.ignoreIndexCheck
        if (check === null || !check(targetValue)) {
            this.move2NewIndex(newKey, targetValue)
        }

        updateKeyCb()
    }

    fun insertByKey(targetValue: V) {
        val key = this.fetch1Key(targetValue)
        val check = this.ignoreIndexCheck
        if (check === null || !check(targetValue)) {
            this.move2NewIndex(key, targetValue)
        }
    }

    private fun move2NewIndex(newKey: K, targetValue: V) {
        this.index[newKey] = targetValue
    }

    fun deleteByKey(targetValue: V): V? {
        val key = this.fetch1Key(targetValue)
        return this.index.remove(key)
    }

    fun deleteWithKey(key: K): V? {
        return this.index.remove(key)
    }

    fun clear() {
        this.index.clear()
    }

    fun findByKey(key: K): V? {
        return this.index[key]
    }

    fun containsKey(key: K): Boolean {
        return this.index.containsKey(key)
    }

    fun getOrPut(key: K, fetchV: () -> V): V {
        var v = findByKey(key)
        if (v != null) {
            return v
        }
        v = fetchV()
        insertByKey(v)
        return v
    }

    /**
     * 筛选出满足条件的
     */
    fun map(selectAndContinue: (v: V) -> Boolean) {
        for ((_, value) in this.index) {
            // 如果checkAndBreak返回false，就跳出循环
            val needContinue = selectAndContinue(value)
            if (!needContinue) {
                break
            }
        }
    }

    /**
     * 遍历所有
     */
    fun foreachItem(selectIt: (v: V) -> Unit) {
        for ((_, value) in this.index) {
            selectIt(value)
        }
    }

    fun fetchAllValues(): List<V> {
        return this.index.values.toList()
    }
}
