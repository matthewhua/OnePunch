/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from select-expression.md by Knit tool. Do not edit.
package io.matt.guide.exampleSelect02

import kotlinx.coroutines.cancelChildren
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.produce
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.selects.select
import kotlinx.coroutines.withTimeoutOrNull

/**
 * select 中的 onReceive ⼦句在已经关闭的通道执⾏会发⽣失败，并导致相应的 select
    抛出异常。我们可以使⽤ onReceiveCatching ⼦句在关闭通道时执⾏特定操作。
 */
suspend fun selectAorB(a: ReceiveChannel<String>, b: ReceiveChannel<String>): String =
    select<String> {
        a.onReceiveCatching { it ->
            val value = it.getOrNull()
            if (value != null) {
                "a -> '$value'"
            } else {
                "Channel 'a' is closed"
            }
        }
        b.onReceiveCatching { it ->
            val value = it.getOrNull()
            if (value != null) {
                "b -> '$value'"
            } else {
                "Channel 'b' is closed"
            }
        }
    }

/**
 * ⾸先， select 偏向于 第⼀个⼦句，当可以同时选到多个⼦句时， 第⼀个⼦句将被选
中。在这⾥，两个通道都在不断地⽣成字符串，因此 a 通道作为 select 中的第⼀个⼦
句获胜。然⽽因为我们使⽤的是⽆缓冲通道，所以 a 在其调⽤ send 时会不时地被挂
起，进⽽ b 也有机会发送。
第⼆个观察结果是，当通道已经关闭时， 会⽴即选择 onReceiveCatching。
 */


fun main() = runBlocking<Unit> {
    val a = produce<String> {
        repeat(4) { send("Hello $it") }
    }
    val b = produce<String> {
        repeat(4) { send("World $it") }
    }
    repeat(8) { // print first eight results
        println(selectAorB(a, b))
    }
    coroutineContext.cancelChildren()  
}    
