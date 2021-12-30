/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from coroutine-context-and-dispatchers.md by Knit tool. Do not edit.
package io.matt.guide.exampleContext01

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.newSingleThreadContext
import kotlinx.coroutines.runBlocking

fun main() = runBlocking<Unit> {
    launch { // context of the parent, main runBlocking coroutine
        println("main runBlocking      : I'm working in thread ${Thread.currentThread().name}")
    }
    launch(Dispatchers.Unconfined) { // not confined -- will work with main thread
        // 协程调度器在调⽤它的线程启动了⼀个协程，但它仅仅只是运 ⾏到第⼀个挂起点。
        println("Unconfined            : I'm working in thread ${Thread.currentThread().name}")
    }
    launch(Dispatchers.Default) { // will get dispatched to DefaultDispatcher 
        println("Default               : I'm working in thread ${Thread.currentThread().name}") //DEFAULT_SCHEDULER_NAME = "DefaultDispatcher"
    }
    val launch = launch(newSingleThreadContext("MyOwnThread")) { // will get its own new thread 为协程的运⾏启动了⼀个线程。
        println("newSingleThreadContext: I'm working in thread ${Thread.currentThread().name}")
    }
}
