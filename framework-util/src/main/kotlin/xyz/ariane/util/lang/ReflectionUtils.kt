package xyz.ariane.util.lang

import com.google.common.collect.HashMultimap
import com.google.common.collect.Multimap
import xyz.ariane.util.io.ClassResolver
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import xyz.ariane.util.dclog.lzDebug
import java.lang.reflect.*
import java.util.*
import java.util.concurrent.ConcurrentHashMap

/**
 * 遍历[rootType]所有直接或间接依赖的其他非抽象类型,遍历规则如下
 *
 * 遇到primitive和java.lang中的类时停止遍历
 * 遇到带泛型参数的类时继续深入遍历其泛型参数制定的类型
 * [Collection]和[Map]的子类必须声明泛型参数,否则抛出[IllegalArgumentException]
 *
 */
class DependencyTreeTraverse(
        val rootType: Type,
        consumer: ((Class<*>) -> Unit)? = null,
        /** 遍历时可获取依赖的深度 */
        consumeWithDepth: ((Class<*>, Int) -> Unit)? = null,
        /** 对于同一个类型，是否对每个不同的依赖深度遍历一次 */
        val consumeSameTypeEachDepth: Boolean = false,
        val fieldFilter: (Field) -> Boolean = { true },
        vararg val scanRootPackages: String
) {

    /** <type, depth> */
    private val visitedTypes: Multimap<Type, Int> = HashMultimap.create()

    private val typeStack: LinkedList<Type> = LinkedList()

    private val logger: Logger = LoggerFactory.getLogger(javaClass)

    private val finalConsume: (Class<*>, Int) -> Unit = consumeWithDepth ?: { c, _ -> consumer?.invoke(c) }

    private val scanRootPackageList: List<String> by lazy(LazyThreadSafetyMode.NONE) { scanRootPackages.asList() }

    companion object {
        /** key: <Type, Package list> */
        private val implementationCache: ConcurrentHashMap<Pair<Type, List<String>>, Collection<Class<*>>> =
                ConcurrentHashMap()
    }

    fun execute() {
        visitedTypes.clear()
        traverseDependencyTree0(rootType, 0)
    }

    private fun traverseDependencyTree0(type: Type, depth: Int) {
        val depthList = visitedTypes.get(type)
        if (depthList.isNotEmpty()) {
            // 这个类型之前执行过traverseDependencyTree0了
            if (consumeSameTypeEachDepth && type is Class<*> && depth !in depthList) {
                finalConsume(type, depth)
                visitedTypes.put(type, depth)
            }
            return
        }

        typeStack.push(type)
        visitedTypes.put(type, depth)
        when (type) {
            is Class<*> -> {
                if (!type.isArray && !type.isPrimitive && !type.isEnum && isAbstractClass(type) && !isJdkContainer(type)) {
                    findImplementations(type)?.forEach { clazz ->
                        traverseDependencyTree0(clazz, depth) // 实现类应该是当前深度
                    }
                } else {
                    finalConsume(type, depth)
                }
                stepIntoClass(type, depth)
            }
            is ParameterizedType -> {
                traverseDependencyTree0(type.rawType, depth) // rawType应该是当前深度
                type.actualTypeArguments.forEach { traverseDependencyTree0(it, depth + 1) }
            }
            is GenericArrayType -> {
                traverseDependencyTree0(type.genericComponentType, depth + 1)
            }
            is WildcardType -> {
                type.lowerBounds.forEach { traverseDependencyTree0(it, depth + 1) }
                type.upperBounds.forEach { traverseDependencyTree0(it, depth + 1) }
            }
        }
        typeStack.pop()
    }

    private fun findImplementations(type: Class<*>): Collection<Class<*>>? {
        logger.lzDebug { "Finding Implementation for type $type" }
        val cacheKey = type to scanRootPackageList
        var implementations = implementationCache[cacheKey]
        if (implementations == null) {
            implementations = ClassResolver<Any>().findImplementations(type, *scanRootPackages).classes
            implementationCache[cacheKey] = implementations
        } else {
            logger.lzDebug { "Hit implementationCache key=$cacheKey" }
        }
        return implementations
    }

    private fun formatTypeStack(): String {
        val sb = StringBuilder().append("Type stack:")
        typeStack.forEach { sb.append("\n\t\t").append(it) }
        return sb.toString()
    }

    fun printStackTrace() = println(formatTypeStack())

    private fun stepIntoClass(clazz: Class<*>, depth: Int) {
        when {
            clazz.isEnum || clazz.isPrimitive -> return
            clazz.name.startsWith("java.lang") -> return
            isJdkContainer(clazz) -> return
            clazz.isArray -> traverseDependencyTree0(clazz.componentType, depth + 1)
            else -> {
                traverseFields(clazz, depth)

                val superclass = clazz.superclass
                if (superclass != null && superclass != Any::class.java) {
                    traverseDependencyTree0(superclass, depth + 1)
                }
            }
        }
    }

    private fun traverseFields(clazz: Class<*>, depth: Int) {
        clazz.declaredFields
                .filter { (it.modifiers and Modifier.STATIC) == 0 && fieldFilter(it) }
                .forEach { field ->
                    val fieldType = field.type
                    val genType = field.genericType
                    if (isJdkContainer(fieldType) && genType !is ParameterizedType) {
                        throw IllegalArgumentException("Collection or Map classes must be generalized. type=$genType, field=$field, class=$fieldType, ${formatTypeStack()}")
                    }
                    if (fieldType.isArray) {
                        traverseDependencyTree0(fieldType, depth + 1)
                    }
                    if (genType is ParameterizedType) {
                        traverseDependencyTree0(genType.rawType, depth + 1)
                    }

                    traverseDependencyTree0(genType, depth + 1)
                }
    }

}

fun isJdkContainer(fClass: Class<*>?): Boolean =
        Collection::class.java.isAssignableFrom(fClass) || Map::class.java.isAssignableFrom(fClass)

fun isAbstractClass(clazz: Class<*>): Boolean = clazz.modifiers and Modifier.ABSTRACT != 0
