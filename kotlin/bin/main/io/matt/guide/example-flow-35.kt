/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow35

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.runBlocking

// Imitate a flow of events
fun events(): Flow<Int> = (1..3).asFlow().onEach { delay(100) }

/**
 * 们需要⼀个类似
addEventListener 的函数，该函数注册⼀段响应的代码处理即将到来的事件，并继续进
⾏进⼀步的处理。onEach 操作符可以担任该⻆⾊。 然⽽， onEach 是⼀个过渡操作
符。我们也需要⼀个末端操作符来收集流。 否则仅调⽤ onEach 是⽆效的
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    events()
        .onEach { event -> println("Event: $event") }
        .collect() // <--- Collecting the flow waits
    println("Done")
}


