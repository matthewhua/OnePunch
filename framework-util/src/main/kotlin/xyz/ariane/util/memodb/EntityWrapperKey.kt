package xyz.ariane.util.memodb

import java.io.Serializable

/**
 * 表示一个[EntityWrapper]的唯一键
 *
 */
class EntityWrapperKey(value: EntityWrapper<*>) {

    val pk: Serializable = value.fetchPrimaryKey()

    val clazz: Class<*> = value.javaClass

    override fun toString(): String = "EntityWrapperKey(pk=$pk, clazz=${clazz.simpleName})"

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other?.javaClass != javaClass) return false

        other as EntityWrapperKey

        if (pk != other.pk) return false
        if (clazz != other.clazz) return false

        return true
    }

    override fun hashCode(): Int {
        var result = pk.hashCode()
        result = 31 * result + clazz.hashCode()
        return result
    }

}