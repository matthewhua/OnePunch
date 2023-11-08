@file:Suppress("unused")

package xyz.ariane.util.distributed

private val worldUnitTimesRange = 1..26

/** 获取平台id */
fun extractPlatformId(worldId: Long): Int = ((worldId / 1000000).toInt())

/** 获取服务器编号 */
fun extractSequenceNo(worldId: Long): Int = (worldId % 10000).toInt()

/** 获取是第几次合服区 */
fun extractUniteTimes(worldId: Long): Int =
    (worldId / 10000L % 100L).toInt().let { if (it in worldUnitTimesRange) it else 0 }

/** 判断是否为合服区 */
fun isUnitedWorld(worldId: Long): Boolean = extractUniteTimes(worldId) > 0

fun notUnitedWorld(worldId: Long): Boolean = !isUnitedWorld(worldId)
