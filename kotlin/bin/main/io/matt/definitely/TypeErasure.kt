package io.matt.definitely


inline fun <reified A, reified B> Pair<*, *>.asPair(): Pair<A, B>? {
    if (first !is A || second !is B) return null
    return first as A to second as B
}

val somePair: Pair<Any?, Any?> = "items" to listOf(1, 2, 3)

val stringToSomething = somePair.asPair<String, Any>()
val stringToInt = somePair.asPair<String, Int>()
val stringToList = somePair.asPair<String, List<*>>()
val stringToStringList = somePair.asPair<String, List<String>>()


fun main() {
    println("stringToSomething = " + stringToSomething)
    println("stringToInt = " + stringToInt)  //null
    println("stringToList = " + stringToList)
    println("stringToStringList = " + stringToStringList)
    //println(stringToStringList?.second?.forEach() {it.length}) // This will throw
}
