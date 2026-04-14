/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow05

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.runBlocking

/**
 * Flow 是⼀种类似于序列的冷流 —
 * 这段 flow 构建器中的代码直到流被收集的时候才运 ⾏.
 */
fun simple(): Flow<Int> = flow {
    println("Flow started")
    for (i in 1..3) {
        delay(100)
        emit(i)
    }
}

/**
 * 这是返回⼀个流的 simple 函数没有标记 suspend 修饰符的主要原因。
 * 通过它⾃ ⼰， simple() 调⽤会尽快返回且不会进⾏任何等待。
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    println("Calling simple function...")
    val flow = simple()
    println("Calling collect...")
    flow.collect { value -> println(value) } 
    println("Calling collect again...")
    flow.collect { value -> println(value) } 
}
