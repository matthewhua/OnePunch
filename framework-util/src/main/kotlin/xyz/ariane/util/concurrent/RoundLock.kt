package xyz.ariane.util.concurrent

import xyz.ariane.util.annotation.AllOpen

@AllOpen
class RoundLock(var num: Int) {
    private var index = 0

    fun fetchIndex(): Int {
        synchronized(this) {
            return (index++) % num
        }
    }
}

@AllOpen
class RoundLockLong(var num: Int) {
    private var index = 0L

    fun fetchIndex(): Long {
        synchronized(this) {
            return (index++) % num
        }
    }
}