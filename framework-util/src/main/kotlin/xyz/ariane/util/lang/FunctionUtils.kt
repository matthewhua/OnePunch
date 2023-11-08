package xyz.ariane.util.lang

import java.util.function.BiFunction
import java.util.function.Function

/**
 * 指定[Class]对象创建一个函数，这个函数用于将对象转型为[expectedClass]类型
 *
 * 如果不能转型抛出[UnexpectedClassException]
 */
@Suppress("UNCHECKED_CAST")
fun <T> castFuncOf(expectedClass: Class<T>): Function<in Any, out T> {
    return Function { obj ->
        when {
            expectedClass.isInstance(obj) -> obj as T
            else -> throw UnexpectedClassException(
                "Expected class is [${expectedClass.name}], but is [${obj.javaClass.name}], instance=$obj",
                obj
            )
        }
    }
}

class UnexpectedClassException(msg: String, val obj: Any) : RuntimeException(msg)

fun <T, U> dummyBiFunc(): BiFunction<T, U, Unit> = BiFunction { _, _ -> Unit }

object DummyRunnable : Runnable {
    override fun run() {
    }
}

