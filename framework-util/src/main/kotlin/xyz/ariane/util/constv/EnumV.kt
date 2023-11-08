package xyz.ariane.util.constv

import com.fasterxml.jackson.databind.util.StdConverter

// 枚举转换成map，提高查找速度
abstract class EnumConverter<in T, R : Enum<R>>(
    private val valueMap: Map<T, R>
) {
    fun fromValue(value: T): R? = valueMap[value]
    fun fromValue(value: T, default: R): R = valueMap[value] ?: default
}

interface EnumConst {
    val v: Int
    val desc: String
}

// 枚举转换成map，提高查找速度
// 同时提供json的反序列化方法，用于快速转换
abstract class EnumConverter4Json<R : Enum<R>>(
    private val valueMap: Map<Int, R>,
    private val clazz: Class<R>
) : StdConverter<String, R>() {
    fun fromValue(value: Int): R? = valueMap[value]
    fun fromValue(value: Int, default: R): R = valueMap[value] ?: default

    override fun convert(value: String?): R {
        if (value == null) {
            throw RuntimeException("解析出错，反序列化的原始值不存在")
        }
        val v = value.toIntOrNull() ?: throw RuntimeException("解析出错，反序列化的原始值不是整数")
        val enumValue = fromValue(v)
        if (enumValue == null) {
            throw RuntimeException("解析出错，反序列化的原始值无法转换为对应的枚举类型。原始值：${value}，对应枚举：${clazz}")
        }
        if (enumValue is EnumConst && enumValue.v == -1) {
            throw RuntimeException("解析出错，转换后的枚举类型为无效值。原始值：${value}，对应枚举：${clazz}")
        }
        return enumValue
    }
}

inline fun <T, reified R : Enum<R>> buildValueMap(keySelector: (R) -> T): Map<T, R> =
    enumValues<R>().associateBy(keySelector)