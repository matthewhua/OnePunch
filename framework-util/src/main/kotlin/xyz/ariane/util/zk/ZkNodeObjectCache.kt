package xyz.ariane.util.zk

import org.apache.curator.framework.recipes.cache.NodeCache
import org.apache.curator.framework.recipes.cache.NodeCacheListener
import xyz.ariane.util.json.mapperUsedInGame

interface ZkNodeObjectWatcher<T : Any> {

    fun onChange(newData: T)

}

/**
 * Zookeeper节点对应数据对象缓存，监听节点数据变化保持同步
 *
 */
abstract class ZkNodeObjectCache<T : Any>(
    val clazz: Class<T>,
) : ZkCache {

    override var initialized = false

    private lateinit var nodeCache: NodeCache

    @Volatile
    protected lateinit var nodeObj: T // 根据数据解析出来的有效实例

    private val watchers = mutableListOf<ZkNodeObjectWatcher<T>>()

    override fun init(zkDao: ZkDao, zkNodePath: String) {
        val zkClient = zkDao.zkClient
        nodeCache = NodeCache(zkClient, zkNodePath)
            .apply { start(true) }

        // 初次更新
        update()

        // 注册变化的监听
        nodeCache.listenable.addListener(NodeCacheListener {
            update()
        })

        initialized = true
    }

    override fun close() {
        nodeCache.close()

        initialized = false
    }

    /**
     * 注册变更观察者
     */
    fun regWatcher(watcher: ZkNodeObjectWatcher<T>) {
        if (!initialized) {
            throw RuntimeException("ZkNodeObjectCache尚未初始化完毕或已结束，无法注册")
        }

        watchers.add(watcher)
    }

    /**
     * 解除变更观察者的注册
     */
    fun unRegWatcher(watcher: ZkNodeObjectWatcher<T>) {
        if (!initialized) {
            throw RuntimeException("ZkNodeObjectCache尚未初始化完毕或已结束，无法解除注册")
        }

        watchers.remove(watcher)
    }

    /**
     * 返回现在的监听列表
     */
    fun fetchWather(): List<ZkNodeObjectWatcher<T>> {
        return watchers
    }

    // mt safe, swap volatile reference only
    private fun update() {
        val zkData: ByteArray? = nodeCache.currentData?.data
        nodeObj =
            if (zkData != null) {
                val obj: T? = mapperUsedInGame.readValue(zkData, clazz)
                obj ?: return
            } else {
                clazz.newInstance()
            }

        onUpdate(nodeObj)

        for (watcher in this.watchers) {
            watcher.onChange(nodeObj)
        }
    }


    abstract fun onUpdate(newData: T)
}