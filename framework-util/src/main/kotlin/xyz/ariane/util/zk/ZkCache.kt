package xyz.ariane.util.zk

interface ZkCache : AutoCloseable {

    var initialized: Boolean

    fun init(zkDao: ZkDao, zkNodePath: String)

}