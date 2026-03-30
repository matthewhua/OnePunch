/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from exception-handling.md by Knit tool. Do not edit.
package io.matt.guide.exampleExceptions06

import kotlinx.coroutines.*
import java.io.IOException


/**
 * 当协程的多个⼦协程因异常⽽失败时， ⼀般规则是“取第⼀个异常”，因此将处理第⼀个
异常。 在第⼀个异常之后发⽣的所有其他异常都作为被抑制的异常绑定⾄第⼀个异常。
 */
@OptIn(DelicateCoroutinesApi::class)
fun main() = runBlocking {
    val handler = CoroutineExceptionHandler { _, exception ->
        println("CoroutineExceptionHandler got $exception")
    }
    val job = GlobalScope.launch(handler) {
        val inner = launch { // all this stack of coroutines will get cancelled
            launch {
                launch {
                    throw IOException() // the original exception
                }
            }
        }
        try {
            inner.join()
        } catch (e: CancellationException) {
            println("Rethrowing CancellationException with original cause")
            throw e // cancellation exception is rethrown, yet the original IOException gets to the handler  
        }
    }
    job.join()
}
