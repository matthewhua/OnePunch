@file:Suppress("MemberVisibilityCanBePrivate")

package xyz.ariane.util.excel

import xyz.ariane.util.annotation.AllOpen
import xyz.ariane.util.i18n.Language
import xyz.ariane.util.io.ClassResolver
import java.util.*

/**
 * 游戏配置,可对应一个配置文件或者一个功能模块
 *
 * 为保持[GameConfigManager]对象可以独立的加载和构建[load],[afterLoadAll]等方法不应该使用依赖全其他全局组件
 *
 */
abstract class GameConfig {

    /**
     * 加载游戏配置
     */
    abstract fun load()

    /**
     * 在所有游戏配置加载完成之后调用,可用于校验、重构数据结构。
     */
    open fun afterLoadAll() = Unit

    /**
     * 所属的管理器对象
     */
    lateinit var manager: GameConfigManager

}

/**
 * 标记此注解的[GameConfig]类将不被加载
 */
@Target(AnnotationTarget.CLASS)
@Retention(AnnotationRetention.RUNTIME)
annotation class DoNotLoadMe

/**
 * 游戏配置管理器
 *
 */
@AllOpen
class GameConfigManager {

    companion object {
        /** 指定前n个加载的配置类和加载顺序,由外部指定 */
        lateinit var firstNOrder: List<Class<out GameConfig>>

        /** 指定配置类afterLoadAll执行顺序的前n个 */
        lateinit var firstNAfterLoadAll: List<Class<out GameConfig>>
    }

    /** 语言 */
    final lateinit var language: Language
        private set

    /** 版本号 */
    final lateinit var version: String
        private set

    /** 所有[GameConfig]对象 */
    protected val configMap: HashMap<Class<out GameConfig>, GameConfig> = hashMapOf()

    /** 是否已经加载完成 */
    protected var loaded = false

    fun init(language: Language, version: String) {
        this.language = language
        this.version = version
    }

    /**
     * 扫描指定包下面的所有[GameConfig]类并实例化,执行加载数据,此方法只能调用一次，第二次调用抛出[IllegalStateException]
     */
    fun scanLoadAll(vararg packages: String) {
        check(!loaded, { "This ${javaClass.simpleName} is already loaded." })

        val configClasses = ClassResolver<GameConfig>()
            .findImplementations(GameConfig::class.java, *packages).classes

        val orderMap: MutableMap<Class<out GameConfig>, Int> = buildOrderMap(firstNOrder)
        configClasses
            .filter { !it.isAnnotationPresent(DoNotLoadMe::class.java) }
            .map { clazz -> clazz.getDeclaredConstructor().newInstance() }
            .sortedBy { config -> orderMap[config.javaClass] ?: Int.MAX_VALUE }
            .forEach { config ->
                config.manager = this
                config.load()
                configMap[config.javaClass] = config
            }

        loaded = true
    }

    private fun buildOrderMap(firstN: List<Class<out GameConfig>>): MutableMap<Class<out GameConfig>, Int> {
        val orderMap: MutableMap<Class<out GameConfig>, Int> = hashMapOf()
        firstN.forEachIndexed { i, clazz ->
            orderMap[clazz] = i
        }
        return orderMap
    }

    fun runAfterLoadAll() {
        check(loaded, { "Call scanLoadAll first." })
        val orderMap = buildOrderMap(firstNAfterLoadAll)
        configMap.values
            .sortedBy { config -> orderMap[config.javaClass] ?: Int.MAX_VALUE }
            .forEach(GameConfig::afterLoadAll)
    }

    @Suppress("UNCHECKED_CAST")
    fun <T : GameConfig> getConfig(clazz: Class<T>): T = configMap[clazz] as T
}

