/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow37

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.runBlocking

fun foo(): Flow<Int> = flow {
    for (i in 1..5) {
        println("Emitting $i") 
        emit(i) 
    }
}

/**
 * 为⽅便起⻅，流构建器对每个发射值执⾏附加的 ensureActive 检测以进⾏取消。 这意
味着从 flow { ... } 发出的繁忙循环是可以取消的
 */
/**
 * 但是，出于性能原因，⼤多数其他流操作不会⾃⾏执⾏其他取消检测。 例如，如果使⽤
IntRange.asFlow 扩展来编写相同的繁忙循环， 并且没有在任何地⽅暂停，那么就没有
取消的检测；
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    foo().collect { value -> 
        if (value == 3) cancel()  
        println(value)
    } 
}
