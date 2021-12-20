/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow30

import kotlinx.coroutines.flow.*
import kotlinx.coroutines.runBlocking

fun simple(): Flow<Int> = flow {
    for (i in 1..3) {
        println("Emitting $i")
        emit(i)
    }
}

/**
 * 我们可以将 catch 操作符的声明性与处理所有异常的期望相结合，将 collect 操作符的代 码块移动到 onEach 中，并将其放到 catch 操作符之前。收集该流必须由调⽤⽆参的
collect() 来触发：
 */
fun main() = runBlocking<Unit> {
    simple()
        .onEach { value ->
            check(value <= 1) { "Collected $value" }                 
            println(value) 
        }
        .catch { e -> println("Caught $e") }
        .collect()
}            
