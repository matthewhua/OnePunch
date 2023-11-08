package xyz.ariane.util.memodb

/**
 * 数据库加载失败的处理接口
 */
interface DbLoadFailedHandle {

    /** 处理[DataContainer.load]的异常 */
    fun handleLoadingException()

    /** 处理[DataContainer.init]的异常 */
    fun handleInitializingException()

}