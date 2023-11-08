package xyz.ariane.util.memodbupgrade

abstract class AbstractDbDataUpgradeHandler<ENTITY_WRAPPER>: IDbDataUpgradeHandler {

    /**
     * 将数据转换为目标类型
     */
    open fun cast(data: Any): ENTITY_WRAPPER {
        return data as ENTITY_WRAPPER
    }

    override fun upgrade(data: Any) {
        val ew = cast(data)

        upgradeEntityWrapper(ew)
    }

    abstract fun upgradeEntityWrapper(ew: ENTITY_WRAPPER)

}