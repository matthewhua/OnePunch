package xyz.ariane.util.lang

class BasicStat {

    var min = 0L
        private set

    var max = 0L
        private set

    var count = 0L
        private set

    var total = 0L
        private set

    fun add(v: Long) {
        min = Math.min(min, v)
        max = Math.max(max, v)
        ++count
        total += v
    }

    fun average(): Double = total.toDouble() / count

    fun clear() {
        min = 0L
        max = 0L
        count = 0L
        total = 0L
    }

}