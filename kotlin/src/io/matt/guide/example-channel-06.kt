/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from channels.md by Knit tool. Do not edit.
package io.matt.guide.exampleChannel06

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.produce
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking

fun main() = runBlocking<Unit> {
    val producer = produceNumbers()
    repeat(5) { launchProcessor(it, producer) }
    delay(950)
   /* repeat(5) {
        for (msg in producer) {
            println("Processor #$it received $msg")
        }
    }*/

    producer.cancel() // cancel producer coroutine and thus kill them all
}

fun CoroutineScope.produceNumbers() = produce<Int> {
    var x = 1 // start from 1
    while (true) {
        send(x++) // produce next
        delay(100) // wait 0.1s
    }
}

fun CoroutineScope.launchProcessor(id: Int, channel: ReceiveChannel<Int>) = launch {
    for (msg in channel) {
        println("Processor #$id received $msg")
    }    
}


/**
    输入结果如下： 且结果不是固定的
    Processor #0 received 1
    Processor #0 received 2
    Processor #1 received 3
    Processor #2 received 4
    Processor #3 received 5
    Processor #4 received 6
    Processor #0 received 7
    Processor #1 received 8
    Processor #2 received 9
 */
