package xyz.ariane.util.memodb

import xyz.ariane.util.memodbupgrade.AbstractVersionDbDataUpgrade
import java.io.Serializable
import java.time.Duration

/**
 * 对应一个entity的内存数据结构.
 *
 *
 * 受[WriteOnlyInMemoryDb4Json]管理，需要使用Kryo序列化，实现类需要注意不要引用外部对象，并保持使用的对象受Kryo支持.
 *
 */
interface EntityWrapper<E : IEntity> {

    /**
     * 获取Entity的主键

     * @return 主键
     */
    fun fetchPrimaryKey(): Serializable

    /**
     * 转为Entity对象，用于同步数据到数据库中
     *
     * **注意：会在IO线程中读取数据，注意线程安全问题，[EntityWrapper]中如果持有entity引用，实现此方法需要复制，不能直接返回内部引用**
     */
    fun toEntity(): E

    /**
     * 使用指定的entity初始化
     */
    fun wrap(upgradeHandlers: List<AbstractVersionDbDataUpgrade>, entity: E)

    /**
     * @return 检查变化的时间间隔
     */
    fun fetchCheckModInterval(): Duration

    /**
     * 计算当前实例的粗检hash值
     */
    fun dirtyHash(): Int = this.hashCode()

    /**
     * 是否是复杂多字段数据，这类数据的脏检查需要针对各个字段独立检查
     */
    fun multiData(): Boolean = multiDataNum() != 1

    /**
     * 字段数据量，如果不是复杂数据，那么是1，如果是复杂数据，那么需要实现类自己控制。
     */
    fun multiDataNum(): Int = 1

    fun multiDatas(): Array<EntityWrapperData> = emptyArray()

    /**
     * 如果是多数据，更新脏记录，方便后续的保存中能看到
     */
    fun updateMultiDirtyRecords(records: Array<ModificationTrackerRecord>): Boolean = true

    /**
     * 是否不可变，既此对象创建后数据内部任何数据不会有变化
     *
     * **注意：这里指的是业务逻辑上不会修改此对象，并不表示此对象是一个immutable object**
     *
     * **注意：返回true的实现类，将不会被跟踪变化**
     */
    val 标记此对象的数据不会被修改_如果返回true变化将不会存库_危险慎用_危险慎用: Boolean get() = false
}
