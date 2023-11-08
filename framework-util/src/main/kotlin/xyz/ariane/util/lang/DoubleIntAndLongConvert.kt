package xyz.ariane.util.lang

import xyz.ariane.util.annotation.AllOpen

var doubleIntAndLongConvert = DoubleIntAndLongConvert()

@AllOpen
class DoubleIntAndLongConvert {

    fun doubleInt2Long(a: Int, b: Int): Long {
        return a.toLong().shl(32) or b.toLong()
    }

    fun long2DoubleInt(v: Long): Pair<Int, Int> {
        return Pair(v.shr(32).toInt(), (v and Long.MAX_VALUE).toInt())
    }

}