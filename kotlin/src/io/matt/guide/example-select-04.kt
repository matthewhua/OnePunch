/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from select-expression.md by Knit tool. Do not edit.
package io.matt.guide.exampleSelect04

import kotlinx.coroutines.*
import kotlinx.coroutines.selects.select
import kotlin.random.Random

fun CoroutineScope.asyncString(time: Int) = async {
    delay(time.toLong())
    "Waited for $time ms"
}


/**
 * 延迟值可以使⽤ onAwait ⼦句查询。 让我们启动⼀个异步函数，它在随机的延迟后会延
    迟返回字符串：
 */
fun CoroutineScope.asyncStringsList(): List<Deferred<String>> {
    val random = Random(3)
    return List(12) { asyncString(random.nextInt(1000)) }
}

/**
 * 现在 main 函数在等待第⼀个函数完成，并统计仍处于激活状态的延迟值的数量。注
意，我们在这⾥使⽤ select 表达式事实上是作为⼀种 Kotlin DSL， 所以我们可以⽤
任意代码为它提供⼦句。在这种情况下，我们遍历⼀个延迟值的队列，并为每个延迟值
提供 onAwait ⼦句的调⽤
 */

fun main() = runBlocking<Unit> {
    val list = asyncStringsList()
    val result = select<String> {
        list.withIndex().forEach { (index, deferred) ->
            deferred.onAwait { answer ->
                "Deferred $index produced answer '$answer'"
            }
        }
    }
    println(result)
    val countActive = list.count { it.isActive }
    println("$countActive coroutines are still active")
}
