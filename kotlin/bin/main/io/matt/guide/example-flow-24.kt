/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow24


import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.runBlocking
import java.lang.System.currentTimeMillis

fun requestFlow(i: Int): Flow<String> = flow {
    emit("$i: First") 
    delay(500) // wait 500 ms
    emit("$i: Second")    
}

/**
 * 并发收集所有传⼊的流，并将它们的值合并到⼀个单独的流，以便尽 快的发射值。 它由 flatMapMerge 与 flattenMerge 操作符实现。
 *
 * 他们都接收可选的⽤于 限制并发收集的流的个数的 concurrency 参数（默认情况下，它等于DEFAULT_CONCURRENCY）
 * 速度上比 flatMapConcat 快乐很多
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    val startTime = currentTimeMillis() // remember the start time 
    (1..3).asFlow().onEach { delay(100) } // a number every 100 ms 
        .flatMapMerge { requestFlow(it) }                                                                           
        .collect { value -> // collect and print 
            println("$value at ${currentTimeMillis() - startTime} ms from start") 
        } 
}

/**
 * 1: First at 152 ms from start
2: First at 256 ms from start
3: First at 369 ms from start
1: Second at 655 ms from start
2: Second at 767 ms from start
3: Second at 880 ms from start
 */