package xyz.ariane.util.math

import xyz.ariane.util.math.Multiplier.Factor
import xyz.ariane.util.math.Multiplier.Fraction

/**
 * 取符号
 */
fun signnum(x: Int): Int = when {
    x > 0 -> 1
    x < 0 -> -1
    else -> 0
}

/**
 * 如果[x]在区间[range]内，返回[x],否则返回与[x]最接近的[range]边界
 */
fun regular(x: Int, range: IntRange): Int {
    require(!range.isEmpty()) { "Range is empty: $range" }
    return when {
        x in range -> x
        x < range.start -> range.start
        x > range.endInclusive -> range.endInclusive
        else -> error("impossible")
    }
}

fun regular(x: Int, min: Int, max: Int): Int = regular(x, min..max)

/** 创建一个long分数 */
infix fun Long.per(denominator: Long): Fraction = Fraction(this, denominator)

/** 创建一个int分数 */
infix fun Int.per(denominator: Int): Fraction = Fraction(this.toLong(), denominator.toLong())

/**
 * [Multiplier]的kotlin封装，使用运算符重载语法糖简化代码
 */
class MULT {

    val m: Multiplier

    constructor(initLong: Long = 1L) {
        m = Multiplier.create(initLong)
    }

    private constructor(multiplier: Multiplier) {
        m = multiplier
    }

    constructor(initInt: Int) : this(initInt.toLong())

    operator fun times(longValue: Long): MULT = apply { m.multi(longValue) }

    operator fun times(intValue: Int): MULT = apply { m.multi(intValue.toLong()) }

    operator fun times(factor: Factor): MULT = apply { m.multi(factor) }

    operator fun div(longValue: Long): MULT = apply { m.multi(1L, longValue) }

    operator fun div(intValue: Int): MULT = apply { m.multi(1L, intValue.toLong()) }

    /** @see Multiplier.multiOnePlus */
    infix fun m1plus(fraction: Fraction): MULT = apply { m.multiOnePlus(fraction.numerator, fraction.denominator) }

    /** @see Multiplier.multiOneMinus */
    infix fun m1minus(fraction: Fraction): MULT = apply { m.multiOneMinus(fraction.numerator, fraction.denominator) }

    fun clear() = m.clear()

    fun calcInt() = m.calcInt()

    fun calcLong() = m.calcLong()

    fun calcDouble() = m.calcDouble()

    fun copy(): MULT = MULT(m.copy())

    override fun toString() = m.toString()
}


