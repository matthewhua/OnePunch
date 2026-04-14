/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from composing-suspending-functions.md by Knit tool. Do not edit.
package io.matt.guide.exampleCompose04

import kotlinx.coroutines.*
import kotlin.system.measureTimeMillis


/**
 * 家人们，这个不推荐使用
 * 如果 val one = somethingUsefulOneAsync() 这⼀⾏和 one.await() 表达式这 ⾥在代码中有逻辑错误， 并且程序抛出了异常以及程序在操作的过程中中⽌，将会发⽣ 什么。
 * 通常情况下，⼀个全局的异常处理者会捕获这个异常，将异常打印成⽇记并报告 给开发者，但是反之该程序将会继续执⾏其它操作。
 */

// note that we don't have `runBlocking` to the right of `main` in this example
fun main() {
    val time = measureTimeMillis {
        // we can initiate async actions outside of a coroutine
        val one = somethingUsefulOneAsync()
        val two = somethingUsefulTwoAsync()
        // but waiting for a result must involve either suspending or blocking.
        // here we use `runBlocking { ... }` to block the main thread while waiting for the result
        runBlocking {
            println("The answer is ${one.await() + two.await()}")
        }
    }
    println("Completed in $time ms") //Completed in 1086 ms
}

@OptIn(DelicateCoroutinesApi::class)
fun somethingUsefulOneAsync() = GlobalScope.async {
    doSomethingUsefulOne()
}

@OptIn(DelicateCoroutinesApi::class)
fun somethingUsefulTwoAsync() = GlobalScope.async {
    doSomethingUsefulTwo()
}

suspend fun doSomethingUsefulOne(): Int {
    delay(1000L) // pretend we are doing something useful here
    return 13
}

suspend fun doSomethingUsefulTwo(): Int {
    delay(1000L) // pretend we are doing something useful here, too
    return 29
}
