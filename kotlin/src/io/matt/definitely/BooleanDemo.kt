package io.matt.definitely

//Boolean has a nullable counterpart Boolean? that also has the null value.

fun main() {
//sampleStart
    val myTrue: Boolean = true
    val myFalse: Boolean = false
    val boolNull: Boolean? = null
    println(myTrue || myFalse)
    println(myTrue && myFalse)
    println(!myTrue) //false
//sampleEnd
}
