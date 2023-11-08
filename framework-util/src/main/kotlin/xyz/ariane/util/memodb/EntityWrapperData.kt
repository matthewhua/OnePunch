package xyz.ariane.util.memodb

interface EntityWrapperData {

    fun dirtyHash(): Int = hashCode()

}