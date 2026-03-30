/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from shared-mutable-state-and-concurrency.md by Knit tool. Do not edit.
package io.matt.guide.exampleSync07

import kotlinx.coroutines.*
import kotlinx.coroutines.channels.actor
import kotlin.system.measureTimeMillis

suspend fun massiveRun(action: suspend () -> Unit) {
    val n = 100  // number of coroutines to launch
    val k = 1000 // times an action is repeated by each coroutine
    val time = measureTimeMillis {
        coroutineScope { // scope for coroutines 
            repeat(n) {
                launch {
                    repeat(k) { action() }
                }
            }
        }
    }
    println("Completed ${n * k} actions in $time ms")    
}

// Message types for counterActor
/**
 * ⼀个 actor 是由协程、 被限制并封装到该协程中的状态以及⼀个与其它协程通信的 通道
组合⽽成的⼀个实体。⼀个简单的 actor 可以简单的写成⼀个函数， 但是⼀个拥有复杂
状态的 actor 更适合由类来表示。

sealed 密封接⼝ can use when
 */
sealed class CounterMsg
object IncCounter : CounterMsg() // one-way message to increment counter
class GetCounter(val response: CompletableDeferred<Int>) : CounterMsg() // a request with reply

// This function launches a new counter actor
fun CoroutineScope.counterActor() = actor<CounterMsg> {
    var counter = 0 // actor state
    for (msg in channel) { // iterate over incoming messages
        when (msg) {
            is IncCounter -> counter++
            is GetCounter -> msg.response.complete(counter)
        }
    }
}

/**
 * actor 本身执⾏时所处上下⽂（就正确性⽽⾔）⽆关紧要。⼀个 actor 是⼀个协程，⽽⼀
个协程是按顺序执⾏的，因此将状态限制到特定协程可以解决共享可变状态的问题。实
际上，actor 可以修改⾃⼰的私有状态， 但只能通过消息互相影响（避免任何锁定）。
actor 在⾼负载下⽐锁更有效，因为在这种情况下它总是有⼯作要做，⽽且根本不需要
切换到不同的上下⽂。



注意，actor 协程构建器是⼀个双重的 produce 协程构建器。⼀个 actor 与它接收
消息的通道相关联，⽽⼀个 producer 与它发送元素的通道相关联。
 */

fun main() = runBlocking<Unit> {
    val counter = counterActor() // create the actor
    withContext(Dispatchers.Default) {
        massiveRun {
            counter.send(IncCounter)
        }
    }
    // send a message to get a counter value from an actor
    val response = CompletableDeferred<Int>()
    counter.send(GetCounter(response))
    println("Counter = ${response.await()}")
    counter.close() // shutdown the actor
}
/**
 * Completed 100000 actions in 306 ms
    Counter = 100000
 */