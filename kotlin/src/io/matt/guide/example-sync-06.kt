/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from shared-mutable-state-and-concurrency.md by Knit tool. Do not edit.
package io.matt.guide.exampleSync06

import kotlinx.coroutines.*
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

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

val mutex = Mutex()
var counter = 0

/**
 * 使⽤永远不会同时执⾏的 关键代码块 来保护共享状态的所有
修改。在阻塞的世界中，你通常会为此⽬的使⽤ synchronized 或者 ReentrantLock 。
在协程中的替代品叫做 Mutex 。它具有 lock 和 unlock ⽅法， 可以隔离关键的部分。关
键的区别在于 Mutex.lock() 是⼀个挂起函数，它不会阻塞线程。
 */
/**
 * 还有 withLock 扩展函数，可以⽅便的替代常⽤的 mutex.lock(); try { …… } finally {
    mutex.unlock() } 模式：
 */
fun main() = runBlocking {
    withContext(Dispatchers.Default) {
        massiveRun {
            // protect each increment with lock
            mutex.withLock {
                counter++
            }
        }
    }
    println("Counter = $counter")
}
/**
 * Completed 100000 actions in 169 ms
Counter = 100000
 */
