/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow08

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.runBlocking

suspend fun performRequest(request: Int): String {
    delay(1000) // imitate long-running asynchronous work
    return "response $request"
}


/**
 * 可以使⽤操作符转换流，就像使⽤集合与序列⼀样。 过渡操作符应⽤于上游流，并返回 下游流。
 * 这些操作符也是冷操作符，就像流⼀样。这类操作符本身不是挂起函数。它运 ⾏的速度很快，返回新的转换流的定义。
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    (1..3).asFlow() // a flow of requests
        .map { request -> performRequest(request) }
        .collect { response -> println(response) }
}
