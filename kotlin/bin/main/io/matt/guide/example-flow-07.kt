/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow07

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.runBlocking

@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    // Convert an integer range to a flow
    (1..3).asFlow().collect { value -> println(value) }
}
