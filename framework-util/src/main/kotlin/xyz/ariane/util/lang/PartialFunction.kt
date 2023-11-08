package xyz.ariane.util.lang

import java.util.function.Function
import java.util.function.Predicate

/**
 * 参考scala的偏函数
 *
 * @see scala.PartialFunction
 * @author jdg
 */
abstract class PartialFunction<X, Y> : Predicate<X>, Function<X, Y> {

    fun isDefinedAt(x: X): Boolean {
        return test(x)
    }

    override infix fun apply(x: X): Y {
        return if (isDefinedAt(x)) {
            applyIfDefined(x)
        } else {
            throw IllegalArgumentException("Value: ($x) isn't supported by this function")
        }
    }

    abstract fun applyIfDefined(x: X): Y

    infix fun orElse(fallback: PartialFunction<X, Y>): PartialFunction<X, Y> {
        val outer: PartialFunction<X, Y> = this
        return object : PartialFunction<X, Y>() {
            override fun test(x: X): Boolean {
                return outer.test(x) || fallback.test(x)
            }

            override fun applyIfDefined(x: X): Y {
                return if (outer.isDefinedAt(x)) {
                    outer.applyIfDefined(x)
                } else {
                    fallback.apply(x)
                }
            }

            override fun apply(x: X): Y {
                return applyIfDefined(x)
            }
        }
    }
}