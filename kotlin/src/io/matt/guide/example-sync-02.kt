/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from shared-mutable-state-and-concurrency.md by Knit tool. Do not edit.
package io.matt.guide.exampleSync02

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

@Volatile // in Kotlin `volatile` is an annotation 
var counter = 0


/**
 * 这段代码运⾏速度更慢了，但我们最后仍然没有得到“Counter = 100000”这个结果，因
为 volatile 变量保证可线性化（这是“原⼦”的技术术语）读取和写⼊变量，但在⼤量动作
（在我们的示例中即“递增”操作）发⽣时并不提供原⼦性。

 */
fun main() = runBlocking {
    withContext(Dispatchers.Default) {
        massiveRun {
            counter++
        }
    }
    println("Counter = $counter")
}

/**
 * Completed 100000 actions in 18 ms
* Counter = 71482
 */
