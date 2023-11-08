package xyz.ariane.util.json

import com.fasterxml.jackson.annotation.JsonAutoDetect
import com.fasterxml.jackson.annotation.PropertyAccessor
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper

// DB差异比较使用的json序列化和反序列化
// 目前还是禁止删字段吧！
val mapperUsedInDB = jacksonObjectMapper().apply {
    //忽略get/set方法，但允许字段属性
    this.setVisibility(PropertyAccessor.SETTER, JsonAutoDetect.Visibility.NONE)
    this.setVisibility(PropertyAccessor.GETTER, JsonAutoDetect.Visibility.NONE)
    this.setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)
}

// 这个方法只在DB序列化时使用！
fun <T> generateJsonBytes4DB(t: T): ByteArray {
    return mapperUsedInDB.writeValueAsBytes(t)
}

// 游戏内使用的json序列化和反序列化
// 目前游戏内还是禁止删字段吧！
val mapperUsedInGame = jacksonObjectMapper().apply {
    //忽略get/set方法，但允许字段属性
    this.setVisibility(PropertyAccessor.SETTER, JsonAutoDetect.Visibility.NONE)
    this.setVisibility(PropertyAccessor.GETTER, JsonAutoDetect.Visibility.NONE)
    this.setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)
}

// 这个Mapper只能在反序列化其他来源的数据时使用，因为这个mapper可以忽略未知字段，所以对方数据结构变了，也不会报错！
val mapperUsedAtHttpReceive = jacksonObjectMapper()
    .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES) // 允许删字段
    .disable(DeserializationFeature.FAIL_ON_MISSING_CREATOR_PROPERTIES)
    .apply {
        //忽略get/set方法，但允许字段属性
        this.setVisibility(PropertyAccessor.SETTER, JsonAutoDetect.Visibility.NONE)
        this.setVisibility(PropertyAccessor.GETTER, JsonAutoDetect.Visibility.NONE)
        this.setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)
    }