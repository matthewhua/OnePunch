/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from shared-mutable-state-and-concurrency.md by Knit tool. Do not edit.
package io.matt.guide.exampleSync03

import kotlinx.coroutines.*
import java.util.concurrent.atomic.AtomicInteger
import kotlin.system.measureTimeMillis

/**
 * 线程安全的数据结构
 * ⼀种对线程、协程都有效的常规解决⽅法，就是使⽤线程安全（也称为同步的、 可线性
化、原⼦）的数据结构，它为需要在共享状态上执⾏的相应操作提供所有必需的同步处
理。
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

val counter = AtomicInteger()

fun main() = runBlocking {
    withContext(Dispatchers.Default) {
        massiveRun {
            counter.incrementAndGet()
        }
    }
    println("Counter = $counter")
}
