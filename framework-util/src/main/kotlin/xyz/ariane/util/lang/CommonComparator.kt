package xyz.ariane.util.lang

import xyz.ariane.util.annotation.AllOpen
import java.util.Comparator

@AllOpen
class AscComparator<T : Comparable<T>> : Comparator<T> {
    override fun compare(o1: T, o2: T): Int {
        if (o1 < o2) {
            return -1
        } else if (o1 > o2) {
            return 1
        }
        return 0
    }
}

@AllOpen
class AscOneComparator<K, T : Comparable<T>>(val fetch: (k: K) -> T) : Comparator<K> {
    override fun compare(o1: K, o2: K): Int {
        if (fetch(o1) < fetch(o2)) {
            return -1
        } else if (fetch(o1) > fetch(o2)) {
            return 1
        }
        return 0
    }
}

@AllOpen
class DescComparator<T : Comparable<T>> : Comparator<T> {
    override fun compare(o1: T, o2: T): Int {
        if (o1 > o2) {
            return -1
        } else if (o1 < o2) {
            return 1
        }
        return 0
    }
}

@AllOpen
class DescOneComparator<K, T : Comparable<T>>(val fetch: (k: K) -> T) : Comparator<K> {
    override fun compare(o1: K, o2: K): Int {
        if (fetch(o1) > fetch(o2)) {
            return -1
        } else if (fetch(o1) < fetch(o2)) {
            return 1
        }
        return 0
    }
}

@AllOpen
class TwoComparator<K>(val mainComparator: Comparator<K>, val secondComparator: Comparator<K>) : Comparator<K> {
    override fun compare(o1: K, o2: K): Int {
        val v = mainComparator.compare(o1, o2)
        if (v != 0) {
            return v
        }
        return secondComparator.compare(o1, o2)
    }
}