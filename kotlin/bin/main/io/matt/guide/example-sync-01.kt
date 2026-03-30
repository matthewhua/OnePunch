/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from shared-mutable-state-and-concurrency.md by Knit tool. Do not edit.
package io.matt.guide.exampleSync01

import kotlinx.coroutines.*
import kotlin.system.measureTimeMillis


/**
 * 协程可⽤多线程调度器（⽐如默认的 Dispatchers.Default）并⾏执⾏。这样就可以提出
所有常⻅的并⾏问题。主要的问题是同步访问共享的可变状态。
 */
suspend fun massiveRun(action: suspend () -> Unit) {
    val n = 100  // number of coroutines to launch
    val k = 1000 // times an action is repeated by each coroutine
    val time = measureTimeMillis {
        coroutineScope { // scope for coroutines 
            repeat(n) {
                launch {
                    repeat(k) { action() }
                }
            }
        }
    }
    println("Completed ${n * k} actions in $time ms")    
}

var counter = 0

fun main() = runBlocking {
    withContext(Dispatchers.Default) {
        massiveRun {
            counter++
        }
    }
    println("Counter = $counter")
}
/**
 * Completed 100000 actions in 15 ms
Counter = 97158
 */
