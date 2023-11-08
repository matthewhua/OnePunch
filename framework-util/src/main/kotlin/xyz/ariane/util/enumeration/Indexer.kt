package xyz.ariane.util.enumeration

import com.google.common.base.Preconditions

import java.util.HashMap

class Indexer<T : IndexedEnum>(values: Array<T>) {

    private val map: MutableMap<Int, T>

    init {
        Preconditions.checkNotNull(values)
        Preconditions.checkArgument(values.size > 0)
        map = HashMap(values.size)
        for (e in values) {
            if (map.containsKey(e.index)) {
                throw RuntimeException("Duplicate index " + e.index + ", class: " + e.javaClass.name)
            }
            map[e.index] = e
        }
    }

    fun valueOf(value: Int): T? {
        return this.map[value]
    }

    fun valueOfChecked(value: Int): T? {
        Preconditions.checkArgument(this.map.containsKey(value), "value:%s", value)
        return this.map[value]
    }
}
