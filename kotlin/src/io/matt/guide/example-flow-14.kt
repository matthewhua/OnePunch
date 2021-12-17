/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow14

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext


/**
 * 然⽽，⻓时间运⾏的消耗 CPU 的代码也许需要在 Dispatchers.Default 上下⽂中执⾏， 并且更新 UI 的代码也许需要在 Dispatchers.Main 中执⾏。通常，withContext ⽤于在
Kotlin 协程中改变代码的上下⽂，但是 flow {...} 构建器中的代码必须遵循上下⽂保 存属性，并且不允许从其他上下⽂中发射（emit）。
 */

fun simple(): Flow<Int> = flow {
    // The WRONG way to change context for CPU-consuming code in flow builder
    withContext(Dispatchers.Default) {
        for (i in 1..3) {
            Thread.sleep(100) // pretend we are computing it in CPU-consuming way 注掉就不会报错
            emit(i) // emit next value
        }
    }
}

@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    simple().collect { value -> println(value) } 
}            
