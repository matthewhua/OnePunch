package xyz.ariane.util.zk

abstract class ZkChildrenCache<T : ZkDomain>() : BaseZkChildrenCache<T, Long>() {

    override fun fetchId(item: T): Long {
        return item.id
    }

}
