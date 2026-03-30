/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from exception-handling.md by Knit tool. Do not edit.
package io.matt.guide.exampleExceptions02

import kotlinx.coroutines.*

/**
 * 将未捕获异常打印到控制台的默认⾏为是可⾃定义的。 根协程中的
CoroutineExceptionHandler 上下⽂元素可以被⽤于这个根协程通⽤的 catch 块，及其
所有可能⾃定义了异常处理的⼦协程。

在 JVM 中可以重定义⼀个全局的异常处理者来将所有的协程通过 ServiceLoader 注册
到 CoroutineExceptionHandler。 全局异常处理者就如同
Thread.defaultUncaughtExceptionHandler ) ⼀样，在没有更多的指定的异常处理者被注
册的时候被使⽤。


CoroutineExceptionHandler 仅在未捕获的异常上调⽤ — 没有以其他任何⽅式处理的异
常。 特别是，所有⼦协程（在另⼀个 Job 上下⽂中创建的协程）委托<!-- 它们的⽗协程
处理它们的异常，然后它们也委托给其⽗协程，以此类推直到根协程， 因此永远不会使
⽤在其上下⽂中设置的 CoroutineExceptionHandler 。 除此之外，async 构建器始终会
捕获所有异常并将其表示在结果 Deferred 对象中， 因此它的
CoroutineExceptionHandler 也⽆效。
 */
@OptIn(DelicateCoroutinesApi::class)
fun main() = runBlocking {
    val handler = CoroutineExceptionHandler { _, exception -> 
        println("CoroutineExceptionHandler got $exception") 
    }
    val job = GlobalScope.launch(handler) { // root coroutine, running in GlobalScope
        throw AssertionError()
    }
    val deferred = GlobalScope.async(handler) { // also root, but async instead of launch
        throw ArithmeticException() // Nothing will be printed, relying on user to call deferred.await()
    }
    joinAll(job, deferred)
}


/**
 * 在监督作⽤域内运⾏的协程不会将异常传播到其⽗协程，并且会从此规则中排除。
本⽂档的另⼀个⼩节——监督提供了更多细节。

 */