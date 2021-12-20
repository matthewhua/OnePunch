/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow32

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.runBlocking

fun simple(): Flow<Int> = (1..3).asFlow()

/**
 * 对于声明式，流拥有 onCompletion 过渡操作符，它在流完全收集时调⽤。 可以使⽤ onCompletion 操作符重写前⾯的示例，并产⽣相同的输出：
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    simple()
        .onCompletion { println("Done") }
        .collect { value -> println(value) }
}            
