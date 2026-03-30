/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow17

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.buffer
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.runBlocking
import kotlin.system.measureTimeMillis

/**
 * 仅仅需要等待 第⼀个数字产⽣的 100 毫秒以及处理每个数字各需花费的 300 毫秒。这种⽅式⼤约花 费了 1000 毫秒来运⾏：
 */
fun simple(): Flow<Int> = flow {
    for (i in 1..3) {
        delay(100) // pretend we are asynchronously waiting 100 ms
        emit(i) // emit next value
    }
}

@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    val time = measureTimeMillis {
        simple()
            .buffer() // buffer emissions, don't wait
            .collect { value -> 
                delay(300) // pretend we are processing it for 300 ms
                println(value) 
            } 
    }   
    println("Collected in $time ms") //Collected in 1062 ms

}
