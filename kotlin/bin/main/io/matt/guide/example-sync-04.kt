/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from shared-mutable-state-and-concurrency.md by Knit tool. Do not edit.
package io.matt.guide.exampleSync04

import kotlinx.coroutines.*
import kotlin.system.measureTimeMillis

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

val counterContext = newSingleThreadContext("CounterContext")
var counter = 0

/**
 * 这段代码运⾏⾮常缓慢，因为它进⾏了 细粒度 的线程限制。每个增量操作都得使⽤
[withContext(counterContext)] 块从多线程 Dispatchers.Default 上下⽂切换到单线程上
下⽂

Completed 100000 actions in 523 ms
Counter = 100000

 */
fun main() = runBlocking {
    withContext(Dispatchers.Default) {
        massiveRun {
            // confine each increment to a single-threaded context
            // 将每次⾃增限制在单线程上下⽂中
            withContext(counterContext) {
                counter++
            }
        }
    }
    println("Counter = $counter")
}
