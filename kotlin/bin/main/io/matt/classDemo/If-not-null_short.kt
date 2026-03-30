package io.matt.classDemo

import java.io.File

/**
 * if-not-null 缩写
 */
fun ifNotnull()
{
    val files = File("Test.text").listFiles()
    println(files?.size) // 如果 files 不是 null，那么输出其⼤⼩（size）

}

/**
 * if-not-null-else 缩写
 */
fun ifNotnullElse()
{
    val files = File("Test.text").listFiles()
    println(files?.size ?: "empty") // // 如果 files 为 null，那么输出“empty”
}

// if null 执⾏⼀个语句
fun ifNullExecute()
{
    val values = hashMapOf<Any, Any>()
    val email = values["email"] ?: throw IllegalStateException("Email is missing!")
}

//在可能会空的集合中取第⼀元素
fun ifNullExecuteGetFist()
{
    val emails = listOf<Int>()
    emails.firstOrNull() ?: 1
}

//if not null 执⾏代码
fun ifNotNullExecute(){
    val values = hashMapOf<Any, Any>()
    val value = values[11]
    value?.let {
        //代码会执⾏到此处, 假如data不为null
    }

    val defaultValue = 1
    //映射可空值（如果⾮空的话）
    val mapped = value?.let {
        transform(it as String)
    } ?: defaultValue
}


fun transform(color: String): Int = when (color) {
    "Red" -> 0
    "Green" -> 1
    "Blue" -> 2
    else -> throw IllegalArgumentException("Invalid color param value")
}
