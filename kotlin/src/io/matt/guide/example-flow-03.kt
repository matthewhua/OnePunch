/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow03


import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking

/**
 * 挂起函数
 * 然⽽，计算过程阻塞运⾏该代码的主线程。 当这些值由异步代码计算时，我们可以使⽤
    suspend 修饰符标记函数 simple ， 这样它就可以在不阻塞的情况下执⾏其⼯作并将 结果作为列表返回：
 */
suspend fun simple(): List<Int> {
    delay(1000) // pretend we are doing something asynchronous here
    return listOf(1, 2, 3)
}

fun main() = runBlocking<Unit> {
    simple().forEach { value -> println(value) } 
}
