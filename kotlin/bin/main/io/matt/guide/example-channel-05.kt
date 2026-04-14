/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from channels.md by Knit tool. Do not edit.
package io.matt.guide.exampleChannel05

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.cancelChildren
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.produce
import kotlinx.coroutines.runBlocking
import kotlin.math.sqrt

var isRunning  = true

fun main() = runBlocking {
    var cur = numbersFrom(2)
    repeat(10) {
        val prime = cur.receive()
        println("prime 为 $prime")
        cur = filter(cur, prime)
    }
    isRunning = false
    coroutineContext.cancelChildren() // cancel all children to let main finish
}

fun CoroutineScope.numbersFrom(start: Int) = produce<Int> {
    var x = start
    while (isRunning){
        send(x++)
    } // infinite stream of integers from start
}

fun CoroutineScope.filter(numbers: ReceiveChannel<Int>, prime: Int) = produce<Int> {
    for (x in numbers){
        //println(x)
        if (x % prime != 0)
            send(x)
    }

}


fun prime(a: Int) : Boolean{
    if(a <= 3) return a > 1;
    for (x in 2 ..sqrt(a.toDouble()).toInt()){
        if ( a % x == 0 ) return false;
    }
    return true;
}


fun isPrime(){
    val prime = prime(17)
    println(prime)
}



