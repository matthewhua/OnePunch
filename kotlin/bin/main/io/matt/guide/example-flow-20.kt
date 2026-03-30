/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow20

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.zip
import kotlinx.coroutines.runBlocking

/**
 * 就像 Kotlin 标准库中的 Sequence.zip 扩展函数⼀样， 流拥有⼀个 zip 操作符⽤于组合 两个流中的相关值：
 */
@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    val nums = (1..3).asFlow() // numbers 1..3
    val strs = flowOf("one", "two", "three") // strings 
    nums.zip(strs) { a, b -> "$a -> $b" } // compose a single string
        .collect { println(it) } // collect and print
}
