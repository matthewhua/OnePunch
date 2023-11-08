package xyz.ariane.util.math

import java.math.RoundingMode
import java.text.DecimalFormat
import kotlin.math.log10
import kotlin.math.pow

infix fun Long.x(that: Long): Long = Math.multiplyExact(this, that)

infix fun Int.x(that: Int): Int = Math.multiplyExact(this, that)

fun Long.toSafeInt(): Int {
    return when {
        this > Int.MAX_VALUE -> Int.MAX_VALUE
        this < Int.MIN_VALUE -> Int.MIN_VALUE
        else -> this.toInt()
    }
}


/**
 * byte[]转int
 * @return Int
 */
fun ByteArray.readInt32(): Int {
    return (this[3].toInt()) and 0xFF or (
            (this[2].toInt()) and 0xFF shl 8) or (
            (this[1].toInt()) and 0xFF shl 16) or (
            (this[0].toInt()) and 0xFF shl 24)
}

/**
 * int转byte[]
 * @return ByteArray
 */
fun Int.readByteArray(): ByteArray {
    return byteArrayOf(
        (this shr 24 and 0xFF).toByte(),
        (this shr 16 and 0xFF).toByte(),
        (this shr 8 and 0xFF).toByte(),
        (this and 0xFF).toByte()
    )
}


/**
 * 这是很神奇的一段代码
 *
 * @param updateTime
 * @return
 */
fun Long.toRedisRankScore(updateTime: Long): Double {
    return this + 1 - updateTime / 10.0.pow(log10(updateTime.toDouble()).toInt() + 1.toDouble())
}

/**
 * 对入参保留最多两位小数(舍弃末尾的0)，如:
 * 3.345->3.34
 * 3.40->3.4
 * 3.0->3
 */
fun Double.getNoMoreThanTwoDigits(): String {
    val format = DecimalFormat("0.##")
    // 未保留小数的舍弃规则，RoundingMode.FLOOR表示直接舍弃。
    format.roundingMode = RoundingMode.FLOOR
    return format.format(this)
}