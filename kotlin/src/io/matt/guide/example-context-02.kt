/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from coroutine-context-and-dispatchers.md by Knit tool. Do not edit.
package io.matt.guide.exampleContext02

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking

fun main() = runBlocking<Unit> {
    launch(Dispatchers.Unconfined) { // not confined -- will work with main thread
        /**
         * 另⼀⽅⾯，该调度器默认继承了外部的 CoroutineScope。 runBlocking 协程的默认调度 器，
         * 特别是， 当它被限制在了调⽤者线程时，继承⾃它将会有效地限制协程在该线程运 ⾏并且具有可预测的 FIFO 调度。
         *
         * 所以，该协程的上下⽂继承⾃ runBlocking {...} 协程并在 main 线程中运⾏，当
        delay 函数调⽤的时候，⾮受限的那个协程在默认的执⾏者线程中恢复执⾏。
         */
        println("Unconfined      : I'm working in thread ${Thread.currentThread().name}")
        delay(500)
        println("Unconfined      : After delay in thread ${Thread.currentThread().name}")
    }
    launch { // context of the parent, main runBlocking coroutine
        println("main runBlocking: I'm working in thread ${Thread.currentThread().name}")
        delay(1000)
        println("main runBlocking: After delay in thread ${Thread.currentThread().name}")
    }
}
