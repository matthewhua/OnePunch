/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from coroutine-context-and-dispatchers.md by Knit tool. Do not edit.
package io.matt.guide.exampleContext11

import kotlinx.coroutines.*

val threadLocal = ThreadLocal<String?>() // declare thread-local variable

/**
 * ThreadLocal ， asContextElement 扩展函数在这⾥会充当救兵。它创建了额外的上下 ⽂元素， 且保留给定 ThreadLocal 的值，并在每次协程切换其上下⽂时恢复它。
 */
fun main() = runBlocking<Unit> {
    threadLocal.set("main")
    println("Pre-main, current thread: ${Thread.currentThread()}, thread local value: '${threadLocal.get()}'")
    val job = launch(Dispatchers.Default + threadLocal.asContextElement(value = "launch")) {
        println("Launch start, current thread: ${Thread.currentThread()}, thread local value: '${threadLocal.get()}'")
        yield()
        println("After yield, current thread: ${Thread.currentThread()}, thread local value: '${threadLocal.get()}'")
    }
    job.join()
    println("Post-main, current thread: ${Thread.currentThread()}, thread local value: '${threadLocal.get()}'")
}
