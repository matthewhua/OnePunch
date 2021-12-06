package io.matt

/**
 * @author Matthew
 * @date 2021-12-07 0:05
 */
fun main()
{

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
            .map { it.toUpperCase() }
            .forEach { (println(it)) }


}