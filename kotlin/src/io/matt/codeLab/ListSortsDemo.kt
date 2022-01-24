package io.matt.codeLab

import org.junit.Assert.assertTrue
import org.junit.Test


class TestClass{

    @Test
    fun listSortDesc(){
        val listOf = listOf(8, 1, 2, 3, 4)
        listOf.sortedBy{ it }
        assertTrue(listOf.size == 5)
    }
}
