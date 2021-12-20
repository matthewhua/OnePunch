/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow22


import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.runBlocking
import java.lang.System.currentTimeMillis


/**
 * 当流表示⼀个变量或操作的最新值时（请参阅相关⼩节 conflation），可能需要执⾏计 算，这依赖于相应流的最新值，并且每当上游流产⽣值的时候都需要重新计算。
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    val nums = (1..3).asFlow().onEach { delay(300) } // numbers 1..3 every 300 ms
    val strs = flowOf("one", "two", "three").onEach { delay(400) } // strings every 400 ms   不需要同步
    val startTime = currentTimeMillis() // remember the start time 
    nums.combine(strs) { a, b -> "$a -> $b" } // compose a single string with "combine"
        .collect { value -> // collect and print 
            println("$value at ${currentTimeMillis() - startTime} ms from start") 
        } 
}
