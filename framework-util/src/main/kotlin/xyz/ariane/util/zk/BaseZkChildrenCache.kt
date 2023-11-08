package xyz.ariane.util.zk

import org.apache.curator.framework.recipes.cache.ChildData
import org.apache.curator.framework.recipes.cache.PathChildrenCache
import org.apache.curator.framework.recipes.cache.PathChildrenCacheEvent
import org.apache.curator.framework.recipes.cache.PathChildrenCacheListener

abstract class BaseZkChildrenCache<T : ZkDomain, ID> : ZkCache {

    override var initialized = false

    var tplMgr = ZkTemplateMgr<T, ID> { fetchId(it) } // 模板

    private lateinit var childrenCache: PathChildrenCache

    abstract fun fetchId(item: T): ID

    override fun init(zkDao: ZkDao, zkNodePath: String) {
        childrenCache = PathChildrenCache(
            zkDao.zkClient,
            zkNodePath,
            true
        ).apply {
            start(PathChildrenCache.StartMode.BUILD_INITIAL_CACHE)
        }

        // 添加已存在的节点
        childrenCache.currentData.forEach {
            val dataStr = it.data.toString(charset = Charsets.UTF_8)

            val template = parse(it.path, dataStr)
            if (template != null) {
                tplMgr.addOrUpdateTemplate(template)

                dealInit(template, dataStr)
            }
        }

        // 监听变化，并处理
        childrenCache.listenable.addListener(PathChildrenCacheListener { _, event ->
            val childData = event.data
            if (childData == null) {
                return@PathChildrenCacheListener
            }

            val dataStr = childData.data.toString(charset = Charsets.UTF_8)

            when (event.type) {
                PathChildrenCacheEvent.Type.CHILD_ADDED -> {
                    handleAdd(zkNodePath, childData, dataStr)
                }
                PathChildrenCacheEvent.Type.CHILD_UPDATED -> {
                    handleUpdate(zkNodePath, childData, dataStr)
                }
                PathChildrenCacheEvent.Type.CHILD_REMOVED -> {
                    handleRemove(zkNodePath, childData, dataStr)
                }
                else -> return@PathChildrenCacheListener
            }
        })

        initialized = true
    }

    override fun close() {
        if (this::childrenCache.isInitialized) {
            childrenCache.close()
        }
    }

    abstract fun parse(path: String, data: String): T?

    abstract fun dealInit(tpl: T, dataStr: String)

    open fun handleAdd(zkNodePath: String, childData: ChildData, dataStr: String) {
        val template = parse(childData.path, dataStr)
        if (template != null) {
            tplMgr.addOrUpdateTemplate(template)

            dealAdd(template, dataStr)
        }
    }

    abstract fun dealAdd(tpl: T, dataStr: String)

    open fun handleUpdate(zkNodePath: String, childData: ChildData, dataStr: String) {
        val template = parse(childData.path, dataStr)
        if (template != null) {
            tplMgr.addOrUpdateTemplate(template)

            dealUpdate(template)
        }
    }

    abstract fun dealUpdate(tpl: T)

    open fun handleRemove(zkNodePath: String, childData: ChildData, dataStr: String) {
        val template = parse(childData.path, dataStr)
        if (template != null) {
            tplMgr.removeTemplate(template)

            dealDelete(template)
        }
    }

    abstract fun dealDelete(tpl: T)
}