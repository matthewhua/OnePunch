package xyz.ariane.util.lang

import java.util.*

fun byteArray2String(bytes: ByteArray): String {
    return Base64.getEncoder().encodeToString(bytes)
}

fun string2ByteArray(str: String): ByteArray {
    return Base64.getDecoder().decode(str)
}