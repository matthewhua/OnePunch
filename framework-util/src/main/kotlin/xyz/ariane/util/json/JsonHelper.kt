package xyz.ariane.util.json

import com.fasterxml.jackson.annotation.JsonAutoDetect
import com.fasterxml.jackson.annotation.PropertyAccessor
import com.fasterxml.jackson.databind.MapperFeature
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue

fun <T> toJson(t: T): String {
    return mapperUsedInGame.writeValueAsString(t)
}

fun <T> toBytesJson(t: T): ByteArray {
    return mapperUsedInGame.writeValueAsBytes(t)
}

inline fun <reified T : Any> toObj(json: String): T {
    return mapperUsedInGame.readValue(json)
}

inline fun <reified T : Any> toObj(json: ByteArray): T {
    return mapperUsedInGame.readValue(json)
}

inline fun <reified T : Any> receive2Obj(json: String): T {
    return mapperUsedAtHttpReceive.readValue(json)
}

inline fun <reified T : Any> receive2Obj(json: ByteArray): T {
    return mapperUsedAtHttpReceive.readValue(json)
}

/**
 * mgr 的api使用 序列化字属性按字段名排序
 */
val mapperUsedInApi = jacksonObjectMapper().apply {
    //忽略get/set方法，但允许字段属性
    this.setVisibility(PropertyAccessor.SETTER, JsonAutoDetect.Visibility.NONE)
    this.setVisibility(PropertyAccessor.GETTER, JsonAutoDetect.Visibility.NONE)
    this.setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)
    this.configure(MapperFeature.SORT_PROPERTIES_ALPHABETICALLY, true)
}

fun <T> toJson4Api(t: T): String {
    return mapperUsedInApi.writeValueAsString(t)
}

// ================== Tree Mode ==================
// 可能抛出异常，逻辑中不要用
fun toTree(json: String): KtJsonNode {
    return KtJsonNode(mapperUsedInGame.readTree(json))
}

fun toTree(json: ByteArray): KtJsonNode {
    return KtJsonNode(mapperUsedInGame.readTree(json))
}

/*inline fun <reified T : Any> toObj(node: TreeNode): T {
    return mapperUsedInGame.treeToValue(node)
}*/

