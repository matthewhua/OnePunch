package xyz.ariane.util.lang

import com.google.common.base.Joiner
import java.net.URLEncoder
import java.util.*
import java.util.concurrent.TimeUnit

fun isNullOrBlank(s: String?): Boolean = s.isNullOrBlank()

fun notNullOrBlank(s: String?): Boolean = !s.isNullOrBlank()

inline fun String.ifNotNullOrBlank(access: (String) -> Unit) {
    if (notNullOrBlank(this)) {
        access(this)
    }
}

val CHINESE_SENTENCE_REGEX = Regex("^[\u4e00-\u9fa5]+$")

fun onlyContainsCn(str: String): Boolean {
    return str.matches(CHINESE_SENTENCE_REGEX)
}

/** 匹配所有双字节字符，比如中日韩、全角 */
val DOUBLE_BYTE_REGEX = Regex("[^\\x00-\\xff]")

val PRINTABLE_REGEX = Regex("\\p{Print}")

fun lengthOfDoublingEnglishChars(str: String): Int {
    return str.replace(PRINTABLE_REGEX, "**").length
}

/** 双字节字符 算2个长度 */
fun lengthOfDoublingDoubleByteChars(str: String): Int {
    return str.replace(DOUBLE_BYTE_REGEX, "**").length
}

fun String.splitByComma(): List<String> = this.split(',', '，')

fun Iterable<String>.joinL(): String = joinToString(separator = "\n")

fun httpJoinParamMapWithUrlEncoding(paramMap: Map<String, String>): String {
    val paramList: ArrayList<String> = arrayListOf()
    paramMap.forEach { name, value ->
        val enc: String = URLEncoder.encode(value, Charsets.UTF_8.name())
        paramList.add(Joiner.on("=").join(name, enc))
    }
    paramList.sort()
    return Joiner.on("&").join(paramList)
}

fun httpBodyMergeOfParamMap(paramMap: Map<String, String>): String {
    val paramList: ArrayList<String> = arrayListOf()
    paramMap.forEach { name, value -> paramList.add(Joiner.on("=").join(name, value)) }
    return Joiner.on("&").join(paramList)
}

/** 是否是自然数 */
fun String.isNumeric(): Boolean = toCharArray().all { Character.isDigit(it) }

private val fourBytesCharRegex = Regex("[\\ud800\\udc00-\\udbff\\udfff\\ud800-\\udfff]")

/**
 *
 * 4字节UTF8字符无法存入Mysql，除非MySQL编码改为 utf8mb4，这里提供一个过滤方法，所有存库的输入字符串需要验证下
 *
 * 替换4字节UTF8字符
 *
 */
@JvmOverloads
fun String.replace4BytesUTF8Char(replace: String = "*"): String = replace(fourBytesCharRegex, replace)

//fun TimeUnit.abbreviate(): String = when (this) {
//    TimeUnit.NANOSECONDS -> "ns"
//    TimeUnit.MICROSECONDS -> "\u03bcs"
//    TimeUnit.MILLISECONDS -> "ms"
//    TimeUnit.SECONDS -> "s"
//    TimeUnit.MINUTES -> "min"
//    TimeUnit.HOURS -> "h"
//    TimeUnit.DAYS -> "d"
//}