package xyz.ariane.util.memodb

import akka.event.LoggingAdapter
import com.google.common.collect.Sets
import org.hibernate.query.Query
import org.hibernate.Session
import org.hibernate.SessionFactory
import org.hibernate.Transaction
import xyz.ariane.util.dclog.lzDebug
import java.io.Serializable

/**
 * 标记持久化实体
 *
 */
interface IEntity : Serializable {
    fun primaryKey(): Serializable
}

/**
 * 数据展开版本，如果数据包含展开属性，就需要这个
 */
interface IWrapEntity : IEntity {
    var aav: Int
}

interface IParallelWrapEntity: IWrapEntity {

    fun parallelIndex(groupSplit: Int): Int

}

/**
 * 通用的Dao
 *
 */
interface CommonDao {

    fun save(entity: IEntity)

    fun <T : IEntity> findById(clazz: Class<T>, id: Serializable): T?

    fun update(entity: IEntity)

    fun saveOrUpdate(entity: IEntity)

    fun delete(entity: IEntity)

    fun <R> findWithTransaction(query: (Session) -> R): R

    fun execWithTransaction(operate: (Session) -> Unit)

    fun close()
}

class CommonDaoHibernate(val sessionFactory: SessionFactory, val useTransaction: Boolean = true) : CommonDao {

    override fun close() {
        sessionFactory.close()
    }

    override fun <R> findWithTransaction(query: (Session) -> R): R {
        val s = sessionFactory.openSession()
        if (useTransaction) {
//            val start = System.nanoTime()
            var tx: Transaction? = null
            try {
                tx = s.beginTransaction()
                tx.setTimeout(300)

                val result = query(s)
//                val start2 = System.nanoTime()
                tx.commit()

//                val end = System.nanoTime()
//                println("CommonDao用时：${(start2 - start) / 1000000} ${(end - start) / 1000000}")

                return result

            } catch (e: Throwable) {
                tx?.rollback()
                throw e
            } finally {
                s.close()
            }
        } else {
            try {
                val result = query(s)

                s.flush()

                return result

            } catch (e: Throwable) {
                throw e
            } finally {
                s.close()
            }
        }

    }

    override fun execWithTransaction(operate: (Session) -> Unit) {
        findWithTransaction { operate(it) }
    }

    @Suppress("UNCHECKED_CAST")
    override fun <T : IEntity> findById(clazz: Class<T>, id: Serializable): T? =
        findWithTransaction { it.get(clazz, id) }

    override fun save(entity: IEntity) {
        execWithTransaction { it.save(entity) }
    }

    override fun update(entity: IEntity) {
        execWithTransaction { it.update(entity) }
    }

    override fun saveOrUpdate(entity: IEntity) {
        execWithTransaction { it.saveOrUpdate(entity) }
    }

    override fun delete(entity: IEntity) {
        execWithTransaction { it.delete(entity) }
    }
}

/**
 * 将list()返回的结果去重返回新的list
 * 用于解决多个分片使用相同数据源时产生重复结果问题
 */
fun <T> Query<T>.listNoDup(logger: LoggingAdapter? = null): List<T> {
    val resultList = list()
    if (resultList.isEmpty()) {
        return emptyList()
    }
    val set: MutableSet<T> = Sets.newLinkedHashSetWithExpectedSize(resultList.size)
    resultList.forEach { entity ->
        if (entity != null && !set.contains(entity)) {
            set.add(entity)
        }
    }

    logger?.lzDebug {
        val first = resultList.firstOrNull()
        "Entity=${first} Result list size=${resultList.size}, return set size=${set.size}"
    }

    return set.toList()
}

fun Session.flushIfReachBatchSize(index: Int, batchSize: Int) {
    if (index != 0 && index % batchSize == 0) {
        flush()
        clear()
    }
}