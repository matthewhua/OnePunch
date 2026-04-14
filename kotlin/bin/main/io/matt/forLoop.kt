package io.matt

/**
 * @author Matthew
 * @date 2021-12-06 23:49
 */
fun maxOf(a: Int, b:Int) = if (a > b) a else b

// when expression

fun describe(obj: Any): String =
        when(obj){
            1           -> "One"
            "Hello"     ->  "Greeting"
            is Long     -> "long"
            !is String  -> "Not a string"
            else        -> "Unknown"
        }


fun main()
{
    println(maxOf(1, 2))

    // for Loop
    val items = listOf("apple", "banana", "kiwifruit")
    for (item in items) {
        println(item)
    }

    for (index in items.indices) {
        println("item at $index is ${items[index]}")
    }

    //whileLoop
    var index = 0
    while (index < items.size){
        println("items at $index is ${items[index]}")
        index++
    }

    println(describe(1.0)) //Not a string
}
