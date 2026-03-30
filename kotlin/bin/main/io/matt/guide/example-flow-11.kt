/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from flow.md by Knit tool. Do not edit.
package io.matt.guide.exampleFlow11

import kotlinx.coroutines.InternalCoroutinesApi
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.runBlocking


@OptIn(InternalCoroutinesApi::class)
fun main() = runBlocking<Unit> {
    val sum = (1..5).asFlow()
        .map { it * it } // squares of numbers from 1 to 5                           
        .reduce { a, b -> a + b } // sum them (terminal operator)
    val count =  (1..5).asFlow()
        .map { it * it } // squares of numbers from 1 to 5
             .collect{ println(it) }
    val list =  (1..5).asFlow()
        .map { it * it } // squares of numbers from 1 to 5
        .toList()
    val set =  (1..5).asFlow()
        .map { it * it } // squares of numbers from 1 to 5
        .toSet()

    println(sum)
    println(count)
    println(list)
    println(set)
}
