package xyz.ariane.util.memodb

data class DirtyCheckRt(
    val wdbType: Int,
    val totalNum: Int, // SS：总数量
    val checkedNum: Int, // SS：检测过的数据数量
    val submitNum: Int, // SS：提交以执行数据库操作的数量
    val dirtyFinishRt: DirtyCheckResult, // SS：检测是以什么样的状态结束的
    val delay3sNum: Int, // SS：检测延迟超过3s的数量
    val costTime: Long // SS：整个检测的耗时，单位ns
)

val emptyDirtyCheckRt = DirtyCheckRt(0, 0, 0, 0, DirtyCheckResult.DIRTY_CHECK_NO_MORE, 0, 0L)