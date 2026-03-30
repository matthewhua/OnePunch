/*
 * Copyright 2016-2020 JetBrains s.r.o. Use of this source code is governed by the Apache 2.0 license.
 */

// This file was automatically generated from coroutines-basics.md by Knit tool. Do not edit.
package io.matt.guide.exampleBasic06

import kotlinx.coroutines.Runnable
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking

fun main() = runBlocking {
    repeat(1_000_000) { // launch a lot of coroutines
        launch {
            delay(5000L)
            print(".")
        }

        Thread(Runnable {  //线程明显很慢
            print("_")
        }).start()
    }


}
