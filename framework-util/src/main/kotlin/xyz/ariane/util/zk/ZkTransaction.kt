package xyz.ariane.util.zk

import org.apache.curator.framework.CuratorFramework
import org.apache.curator.framework.api.transaction.CuratorTransactionFinal
import org.apache.curator.framework.api.transaction.CuratorTransactionResult
import org.slf4j.Logger
import xyz.ariane.util.annotation.AllOpen
import java.util.*

typealias OpType = Int

const val OperationCreate: OpType = 1
const val OperationDelete: OpType = 2
const val OperationSetData: OpType = 3

@AllOpen
class ZkTransaction(val client: CuratorFramework, val logger: Logger) {

    data class TransactionOperation(val op: OpType, val path: String, val data: String = "")

    private val opList = LinkedList<TransactionOperation>()

    fun create(path: String, data: String = "") {
        if (client.checkExists().forPath(path) != null) {
            opList.add(TransactionOperation(OperationSetData, path, data))
            return
        }
        opList.add(TransactionOperation(OperationCreate, path, data))
    }

    fun delete(path: String) {
        opList.add(TransactionOperation(OperationDelete, path))
    }

    fun changeData(path: String, data: String) {
        opList.add(TransactionOperation(OperationSetData, path, data))
    }

    /**
     * 提交
     */
    fun commit(): Collection<CuratorTransactionResult>? {
        if (opList.isEmpty()) {
            return null
        }
        try {
            var transaction = client.inTransaction()
            opList.forEach {
                transaction = when (it.op) {
                    OperationCreate -> {
                        transaction.create().forPath(it.path, it.data.toByteArray()).and()
                    }
                    OperationDelete -> {
                        transaction.delete().forPath(it.path).and()
                    }
                    else -> {
                        transaction.setData().forPath(it.path, it.data.toByteArray()).and()
                    }
                }
            }
            if (transaction is CuratorTransactionFinal) {
                return (transaction as CuratorTransactionFinal).commit()
            }
        } catch (e: Exception) {
            e.printStackTrace()
            logger.error("zk事物提交异常:$e")
        }
        return null
    }
}