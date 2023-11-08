package xyz.ariane.util.math

import com.google.common.base.Preconditions
import com.google.common.collect.Lists
import com.google.common.math.LongMath
import com.google.common.primitives.Ints
import java.util.*
import java.util.stream.Collectors

/**
 * 整数结果乘法器,对分数进行自动约分,尽可能防止溢出
 *
 */
class Multiplier(
        val initValue: Long // 初始值
) {
    /**
     * 分数因数
     */
    private var factors: MutableList<Factor> = Lists.newArrayListWithExpectedSize(3)

    interface Factor {

        /**
         * 分子
         */
        val numerator: Long

        /**
         * 分母
         */
        val denominator: Long
    }

    /**
     * 整数因数
     */
    class IntFactor(override val numerator: Long) : Factor {
        override val denominator: Long
            get() = 1
    }

    /**
     * 分数因数,x/y
     */
    open class Fraction(numerator: Long, denominator: Long) : Factor {

        override val numerator: Long

        override val denominator: Long

        init {
            Preconditions.checkArgument(denominator != 0L, "分母不能为0, 分子:%s", numerator)
            val gcd = LongMath.gcd(Math.abs(numerator), Math.abs(denominator))
            this.numerator = numerator / gcd
            this.denominator = denominator / gcd
        }

    }

    /**
     * (1+x/y) <=> (x+y)/y
     */
    class OnePlusFraction(numerator: Long, denominator: Long) : Fraction(numerator, denominator) {

        override val numerator: Long
            get() = super.numerator + super.denominator

    }

    constructor() : this(1L) {}

    fun copy(): Multiplier {
        val copy = Multiplier(initValue)
        copy.factors = Lists.newArrayList(factors)
        return copy
    }

    fun multi(factor: Factor): Multiplier {
        Preconditions.checkNotNull(factor)
        this.factors.add(factor)
        return this
    }

    /**
     * 乘整数
     *
     * @param longValue 乘数
     * @return self
     */
    fun multi(longValue: Long): Multiplier {
        this.factors.add(IntFactor(longValue))
        return this
    }

    /**
     * 乘分数
     *
     * @param numerator   分子
     * @param denominator 分母
     * @return self
     */
    fun multi(numerator: Long, denominator: Long): Multiplier {
        this.factors.add(Fraction(numerator, denominator))
        return this
    }

    /**
     * *(1+x/y)
     *
     * @param numerator   分子
     * @param denominator 分母
     * @return self
     */
    fun multiOnePlus(numerator: Long, denominator: Long): Multiplier {
        this.factors.add(OnePlusFraction(numerator, denominator))
        return this
    }

    /**
     * *(1-x/y)
     *
     * @param numerator   分子
     * @param denominator 分母
     * @return self
     */
    fun multiOneMinus(numerator: Long, denominator: Long): Multiplier {
        return multiOnePlus(-numerator, denominator)
    }

    /**
     * 计算结果
     *
     * @return 结果值
     */
    fun calcLong(): Long {
        val fraction = calcFraction()
        if (fraction.isPresent) {
            val (first, second) = fraction.get()
            return first / second
        } else {
            return initValue
        }
    }

    /**
     * @return double
     */
    fun calcDouble(): Double {
        val fraction = calcFraction()
        if (fraction.isPresent) {
            val (first, second) = fraction.get()
            return first.toDouble() / second
        } else {
            return initValue.toDouble()
        }
    }

    /**
     * 计算出最终分子分母，Pair中 左分子右分母 不会为空
     *
     * @return Optional
     */
    private fun calcFraction(): Optional<Pair<Long, Long>> {
        if (this.factors.isEmpty()) {
            return Optional.empty()
        }
        var numeratorProduct = this.initValue
        var denominatorProduct = 1L
        for (factor in this.factors) {
            val num = factor.numerator
            if (num == 0L) {
                return Optional.of(Pair(0L, 1L))
            }
            if (numeratorProduct != 0L) {
                if (Math.abs(num) > java.lang.Long.MAX_VALUE / Math.abs(numeratorProduct)) {
                    throw IllegalStateException(String.format("分子溢出:%d,因数:%d,所有因数:%s", numeratorProduct, num, toString()))
                }
            }
            numeratorProduct *= num

            val den = factor.denominator
            if (denominatorProduct != 0L) {
                if (Math.abs(den) > java.lang.Long.MAX_VALUE / Math.abs(denominatorProduct)) {
                    throw IllegalStateException(String.format("分母溢出:%d,因数:%d,所有因数:%s", numeratorProduct, num, toString()))
                }
            }
            denominatorProduct *= den
        }
        return Optional.of(Pair(numeratorProduct, denominatorProduct))
    }

    fun calcInt(): Int {
        return Ints.checkedCast(calcLong())
    }

    fun clear() {
        this.factors.clear()
    }

    override fun toString(): String {
        val factorStrs = this.factors.stream()
                .map { factor -> "(" + factor.numerator + "/" + factor.denominator + ")" }
        return factorStrs.collect(Collectors.joining())
    }

    companion object {

        fun create(): Multiplier {
            return Multiplier()
        }

        fun create(initValue: Long): Multiplier {
            return Multiplier(initValue)
        }
    }

}
