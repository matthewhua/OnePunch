package xyz.ariane.util.lang

import xyz.ariane.util.annotation.AllOpen
import java.util.*

var numAndStrConvert = NumAndStrConvert()

@AllOpen
class NumAndStrConvert {
    fun num10to62Str(number: Long): String {
        val charSet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".toCharArray()
        var rest = number
        val stack = Stack<Char>()
        val result = StringBuilder(0)
        while (rest != 0L) {
            stack.add(charSet[(rest - rest / 62 * 62).toInt()])
            rest /= 62
        }
        while (!stack.isEmpty()) {
            result.append(stack.pop())
        }

        return result.toString()
    }

    fun num10to36Str(number: Long): String {
        val charSet = "0123456789abcdefghijklmnopqrstuvwxyz".toCharArray()
        var rest = number
        val stack = Stack<Char>()
        val result = StringBuilder(0)
        while (rest != 0L) {
            stack.add(charSet[(rest - rest / 36 * 36).toInt()])
            rest /= 36
        }
        while (!stack.isEmpty()) {
            result.append(stack.pop())
        }

        return result.toString()
    }

    fun str62tonum10(str: String): Long? {
        val charList = str.toCharArray().reversed()
        val charSet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".toCharArray()
        var index = 1L
        var number = 0L
        for (c in charList) {
            if (c !in '0'..'9' && c !in 'a'..'z' && c !in 'A'..'Z') {
                return null
            }

            number += charSet.indexOf(c) * index
            index *= 36
        }
        return number
    }

    fun str36tonum10(str: String): Long? {
        val charList = str.toCharArray().reversed()
        val charSet = "0123456789abcdefghijklmnopqrstuvwxyz".toCharArray()
        var index = 1L
        var number = 0L
        for (c in charList) {
            if (c !in '0'..'9' && c !in 'a'..'z') {
                return null
            }
            number += charSet.indexOf(c) * index
            index *= 36
        }
        return number
    }

    // 校验玩家Id是否含有非法字符
    fun checkConvert(playerId: String): Boolean {
        val charList = playerId.toCharArray().reversed()
        for (c in charList) {
            if (c !in '0'..'9' && c !in 'a'..'z') {
                return false
            }
        }
        return true
    }

    // 校验是否满足36字符压缩玩家Id的长度(仅仅是长度校验，并不说明是有效的playerId)
    fun checkStr36(playerId: String): Boolean {
        // 36字符转换后的playerId都是10字符的
        return playerId.trim().length == 10
    }
}