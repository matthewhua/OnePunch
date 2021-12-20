/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow19


import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.runBlocking
import kotlin.system.measureTimeMillis

fun simple(): Flow<Int> = flow {
    for (i in 1..3) {
        delay(100) // pretend we are asynchronously waiting 100 ms
        emit(i) // emit next value
    }
}

fun main() = runBlocking<Unit> { 
    val time = measureTimeMillis {
        simple()
            .collectLatest { value -> // cancel & restart on the latest value
                println("Collecting $value") 
                delay(300) // pretend we are processing it for 300 ms
                println("Done $value")   //最后只执行了最后一个
            } 
    }   
    println("Collected in $time ms") // 100 + 100 + 100 + 300
}

/**
 * 由于 collectLatest 的函数体需要花费 300 毫秒，但是新值每 100 秒发射⼀次，我们看 到该代码块对每个值运⾏，但是只收集最后⼀个值
 */