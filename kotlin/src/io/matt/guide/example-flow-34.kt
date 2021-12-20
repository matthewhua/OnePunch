/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package kotlinx.coroutines.guide.exampleFlow34

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.runBlocking

fun simple(): Flow<Int> = (1..3).asFlow()

@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    simple()
        /**
         * 与 catch 操作符的另⼀个不同点是 onCompletion 能观察到所有异常并且仅在上游流成 功完成（没有取消或失败）的情况下接收⼀个 null 异常。
         */
        .onCompletion { cause -> println("Flow completed with $cause") }
        .collect { value ->
            check(value <= 1) { "Collected $value" }
            println(value) 
        }
}
