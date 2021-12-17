/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from coroutine-context-and-dispatchers.md by Knit tool. Do not edit.
package io.matt.guide.exampleContext05

import kotlinx.coroutines.Job
import kotlinx.coroutines.runBlocking

/**
 * 请注意，CoroutineScope 中的 isActive 只是 coroutineContext[Job]?.isActive == true
的⼀种⽅便的快捷⽅式。
 */
fun main() = runBlocking<Unit> {
    println("My job is ${coroutineContext[Job]}")
}
