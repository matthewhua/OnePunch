package xyz.ariane.util.memodbupgrade

/**
 * 数据库序列化字段更新处理
 */
interface IDbDataUpgradeHandler {

    fun upgrade(data: Any)

}