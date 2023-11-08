package xyz.ariane.util.zk

import org.apache.curator.framework.CuratorFramework
import org.apache.curator.framework.recipes.atomic.DistributedAtomicLong
import org.apache.curator.framework.recipes.shared.SharedCount
import org.apache.curator.retry.RetryNTimes
import org.apache.zookeeper.CreateMode
import org.apache.zookeeper.KeeperException
import org.slf4j.Logger
import xyz.ariane.util.json.KtJsonNode
import xyz.ariane.util.json.toObj
import xyz.ariane.util.json.toTree

class ZkDao(val zkClient: CuratorFramework, val logger: Logger) {

    /**
     * 生成特定类型的Id值
     *
     * @param path Zk计数器路径
     * @param seed 种子
     * @return 返回ID值
     */
    fun generateId(path: String, seed: Int): Int? {
        return zkIdGen(zkClient, path, seed)
    }

    /**
     * 生成zkId
     */
    private fun zkIdGen(client: CuratorFramework, path: String, seed: Int = 0, retry: Int = 20): Int? {
//        val counter = DistributedAtomicLong(client, path, RetryNTimes(5, 0))
//
//        val sequence = counter.increment()
//        if (!sequence.succeeded()) {
//            return null
//        }
//        return sequence.postValue()

        // 共享计数器
        val counter = SharedCount(client, path, seed)
        try {
            counter.start()
            for (i in 1..retry) {
                val newCount: Int
                val rt = counter.versionedValue.let {
                    newCount = it.value + 1
                    counter.trySetCount(it, newCount)
                }
                if (!rt) {
                    continue
                }
                return newCount
            }
            logger.warn("重试${retry}次之后仍然无法设置新的Id")

        } catch (e: Exception) {
            e.printStackTrace()
            logger.error("生成zkId异常:$e")
        } finally {
            counter.close()
        }

        return null
    }

    /**
     * 找到特定路径节点的数据
     */
    fun findNodeData(path: String): String? {
        val nodeData =
            try {
                zkClient.data.forPath(path)
            } catch (noNodeE: KeeperException.NoNodeException) {
                //节点不存在
                return null
            }
        return nodeData.toString(charset = Charsets.UTF_8)
    }

    inline fun <reified T : Any> findSpecTypeNodeData(path: String): T? {
        val nodeData =
            try {
                zkClient.data.forPath(path)
            } catch (noNodeE: KeeperException.NoNodeException) {
                //节点不存在
                return null
            }
        val dataStr = nodeData.toString(charset = Charsets.UTF_8)
        if (dataStr == "") {
            return null
        }
        return toObj(dataStr)
    }

    /**
     * 使用Tree Mode获取Zk数据
     */
    fun findTreeTypeNodeData(path: String): KtJsonNode? {
        val nodeData =
            try {
                zkClient.data.forPath(path)
            } catch (noNodeE: KeeperException.NoNodeException) {
                //节点不存在
                return null
            }
        val dataStr = nodeData.toString(charset = Charsets.UTF_8)
        if (dataStr == "") {
            return null
        }
        return toTree(dataStr)
    }

    /**
     * 找到所有子节点的名字
     */
    fun findNodeNameOfChildren(path: String): List<String> {
        return try {
            zkClient.children.forPath(path)
        } catch (noNodeE: KeeperException.NoNodeException) {
            //节点不存在
            listOf()
        }
    }

    /**
     * 清理特定路径的节点数据和子节点
     */
    fun clear(path: String): Boolean {
        if (zkClient.checkExists().forPath(path) == null) {
            //节点不存在
            return false
        }

        zkClient.delete().guaranteed().deletingChildrenIfNeeded().forPath(path)
        return true
    }

    /**
     * 创建或更新指定节点的数据
     */
    fun createOrUpdateNode(path: String, domainData: String) {
        val byteData = domainData.toByteArray(charset = Charsets.UTF_8)
        createOrUpdateNode(path, byteData)
    }

    fun createOrUpdateNode(path: String, byteData: ByteArray) {
        if (zkClient.checkExists().forPath(path) == null) {
            //节点不存在直接创建
            zkClient.create().creatingParentsIfNeeded().withMode(CreateMode.PERSISTENT)
                .forPath(path, byteData)
        } else {
            zkClient.setData().forPath(path, byteData)
        }
    }
}