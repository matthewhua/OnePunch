package io.matt

import java.util.*



fun immutableMapDemo(){
    val numbersMap = mapOf("one" to 1, "two" to 2, "three" to 3) // 不可变Map
    println(numbersMap.get("one"))
    println(numbersMap["one"])
    println(numbersMap.getOrDefault("four", 10))
    println(numbersMap["five"]) // null
    println(numbersMap.keys)
    println(numbersMap.values)
    println(numbersMap.entries)
}

fun mutableMapDemo(){
    val mutableMap = mutableMapOf("one" to 1, "shy" to 1314)
    // 添加少数
    mutableMap.put("three" , 3) // Insert
    mutableMap["four"] = 4      //Assign 赋值
    println(mutableMap)

    // 添加多个
    mutableMap.putAll(setOf("matthew" to  6, "vanida" to 7))
    println(mutableMap)
    // plusAssign （ += ） 操作符。
    mutableMap += mapOf("hhh" to 8, "Aloha" to 88)
    println(mutableMap)

    /****************************** 下面是可变Map 的删除操作 ***********************************/
    mutableMap.remove("hhh")
    println(mutableMap)
    mutableMap.remove("Aloha", 8) //删除失败
    mutableMap.remove("Aloha", 88) //删除成功
    println(mutableMap)
    // 开始骚操作
    mutableMap.keys.remove("vanida")
    println(mutableMap)
    mutableMap["test"] = 1
    println(mutableMap)
    getMutableMapDemo(mutableMap)
    mutableMap.values.remove(1) // 把one 删掉了,只删除掉一个
    mutableMap.values.remove(1) // test 删掉了

    println(mutableMap)

}

fun getMutableMapDemo(mutableMap: MutableMap<String, Int>) {
    val filterKeys = mutableMap.filterKeys { it == "one" }
    println("filterKeys $filterKeys")
    val filterValues = mutableMap.filterValues { it < 3 }
    println("filterValues $filterValues")


    // * ""
}


fun listDemo(){
    val listOf = listOf(User2("matthew", 18), User2("Alice", 19))
    val map = listOf.map { it.id }
    println("map的 Id 为 $map, list的 类型为${listOf.javaClass.canonicalName}, map 的类型为 ${map.javaClass.canonicalName}")
}


/**
 * @author Matthew
 * @date 2021-12-07 0:05
 */
fun main()
{
    listDemo()
    val items = listOf("apple", "banana", "kiwifruit")
    for (item in items)
    {
        println(item)
    }

    when{
        "orange" in items -> println("juicy")
        "apple" in items -> println("apple is fine too")
    }

    val fruits = listOf("banana", "avocado", "apple", "kiwifruits")

    fruits
            .filter { it.startsWith("a") }
            .sortedBy { it }
            .map { it.uppercase(Locale.getDefault()) }
            .forEach { (println(it)) }

    println("*************************** 下面是不可变Map **********************************")
    immutableMapDemo()

    println("*************************** 下面是可变Map **********************************")
    mutableMapDemo()
}