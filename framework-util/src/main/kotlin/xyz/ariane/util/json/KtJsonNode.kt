package xyz.ariane.util.json

import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.node.*
import java.util.*

class KtJsonNode(private val node: JsonNode) {

    fun textValue(): String {
        if (!node.isTextual) {
            throw Exception("node.isTextual: ${node.isTextual}")
        }

        return node.textValue()
            ?: throw Exception("$node 无法转化为TextValue")
    }

    fun intValue(): Int {
        if (!node.isInt) {
            throw Exception("node.isInt: ${node.isInt}")
        }

        return node.intValue()
    }

    fun longValue(): Long {
        if (!node.isLong) {
            throw Exception("node.isLong: ${node.isLong}")
        }

        return node.longValue()
    }

    fun floatValue(): Float {
        if (!node.isFloat) {
            throw Exception("node.isFloat: ${node.isFloat}")
        }

        return node.floatValue()
    }

    fun toList(): List<KtJsonNode> {
        if (!node.isArray) {
            throw Exception("node.isArray: ${node.isArray}")
        }

        return node.map {
            KtJsonNode(it)
        }
    }


    fun getMapValues(): List<KtJsonNode> {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val list = LinkedList<KtJsonNode>()
        (node as ObjectNode).fields().forEach { (_, node) ->
            list.add(KtJsonNode(node))
        }
        return list
    }

    /**
     * 获取特定名字的Map类型的成员的值。
     */
    fun fetchMapNode(fieldName: String): MutableMap<String, KtJsonNode> {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val map = mutableMapOf<String, KtJsonNode>()
        val fieldNode = (node as ObjectNode).get(fieldName)
            ?: throw Exception("无法在 $node 中找到 $fieldName 字段")
        fieldNode.fields().forEach { (name, node) ->
            map[name] = KtJsonNode(node)
        }
        return map
    }

    /**
     * 获取特定名字的List类型的成员的值。
     * @param fieldName String
     * @return List<KtJsonNode>
     */
    fun fetchListNode(fieldName: String): List<KtJsonNode> {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val getNode = (node as ObjectNode).get(fieldName)
            ?: throw Exception("无法在 $node 中找到 $fieldName 字段")

        return KtJsonNode(getNode).toList()
    }

    /**
     * 获取特定名字的String类型的成员的值。
     * @param fieldName String
     * @return List<KtJsonNode>
     */
    fun fetchTextNode(fieldName: String): String {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val getNode = (node as ObjectNode).get(fieldName)
            ?: throw Exception("无法在 $node 中找到 $fieldName 字段")

        return KtJsonNode(getNode).textValue()
    }

    /**
     * 获取特定名字的Int类型的成员的值
     */
    fun fetchIntNode(fieldName: String): Int {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val getNode = (node as ObjectNode).get(fieldName)
            ?: throw Exception("无法在 $node 中找到 $fieldName 字段")

        return KtJsonNode(getNode).intValue()
    }

    /**
     * 获取特定名字的Long类型的成员的值
     */
    fun fetchLongNode(fieldName: String): Long {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val getNode = (node as ObjectNode).get(fieldName)
            ?: throw Exception("无法在 $node 中找到 $fieldName 字段")

        return KtJsonNode(getNode).longValue()
    }

    /**
     * 获取特定名字的Float类型的成员的值
     */
    fun fetchFloatNode(fieldName: String): Float {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val getNode = (node as ObjectNode).get(fieldName)
            ?: throw Exception("无法在 $node 中找到 $fieldName 字段")

        return KtJsonNode(getNode).floatValue()
    }

    /**
     * 移除特定名字的成员
     */
    fun removeJsonNode(fieldName: String): KtJsonNode {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        val removeNode = (node as ObjectNode).remove(fieldName)
            ?: throw Exception("无法在 $node 中找到 $fieldName 字段")

        return KtJsonNode(removeNode)
    }

    /**
     * 添加到List类型的成员节点中
     */
    fun add(addNode: KtJsonNode) {
        if (!node.isArray) {
            throw Exception("node.isArray: ${node.isArray}")
        }

        (node as ArrayNode).add(addNode.node)
    }

    /**
     * 添加特定名字的Boolean成员节点
     */
    fun addOrModifyJsonNode(fileName: String, value: Boolean) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, KtJsonNode(BooleanNode.valueOf(value)).node)
    }

    /**
     * 添加特定名字的Short成员节点
     */
    fun addOrModifyJsonNode(fileName: String, value: Short) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, KtJsonNode(ShortNode.valueOf(value)).node)
    }

    /**
     * 添加特定名字的Int成员节点
     */
    fun addOrModifyJsonNode(fileName: String, value: Int) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, KtJsonNode(IntNode(value)).node)
    }

    /**
     * 添加特定名字的Long成员节点
     */
    fun addOrModifyJsonNode(fileName: String, value: Long) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, KtJsonNode(LongNode(value)).node)
    }

    /**
     * 添加特定名字的String成员节点
     */
    fun addOrModifyJsonNode(fileName: String, value: String) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, KtJsonNode(TextNode(value)).node)
    }

    /**
     * 添加特定名字的List<T>成员节点
     */
    fun <T> addOrModifyJsonNode(fileName: String, value: List<T>) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, toTree(toJson(value)).node)
    }

    /**
     * 添加特定名字的对象成员节点
     */
    fun addOrModifyJsonNode4Obj(fileName: String, value: Any) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, toTree(toJson(value)).node)
    }

    /**
     * 添加特定名字的成员节点
     */
    fun addOrModifyJsonNode4JsonNode(fileName: String, value: KtJsonNode) {
        if (!node.isObject) {
            throw Exception("node.isObject: ${node.isObject}")
        }

        (node as ObjectNode).set<JsonNode>(fileName, value.node)
    }

    override fun toString(): String {
        return node.toString()
    }
}