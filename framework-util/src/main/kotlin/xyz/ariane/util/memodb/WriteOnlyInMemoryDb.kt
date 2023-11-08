package xyz.ariane.util.memodb

import java.lang.reflect.Field
import java.lang.reflect.Modifier
import java.util.concurrent.TimeUnit

// 脏检查结束结果
enum class DirtyCheckResult(val v: Int) {
    DIRTY_CHECK_OUT_LOOP(0),
    DIRTY_CHECK_NO_MORE(1),
    DIRTY_CHECK_OUT_TIME(2)
}

val TRANSIENT_FILED_FILTER = { field: Field -> field.modifiers and Modifier.TRANSIENT == 0 }

/** 存库操作超时时间(ms) 原始的版本是300秒，这里设置为60秒，缩短超时重试间隔 */
@Volatile
internal var operationTimeoutMillis: Long = 60_000L

/** Only for test */
internal var operationTimeoutSeconds: Long
    get() = TimeUnit.MILLISECONDS.toSeconds(operationTimeoutMillis)
    set(value) {
        operationTimeoutMillis = TimeUnit.SECONDS.toMillis(value)
    }

/** 是否开启打印统计信息 */
@Volatile
internal var turnOnStats = true


