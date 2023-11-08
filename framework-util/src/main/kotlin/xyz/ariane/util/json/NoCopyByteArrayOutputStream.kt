package xyz.ariane.util.json

import java.io.ByteArrayOutputStream

// 能直接获取到ByteArrayOutputStream的缓冲区，用于避免内存复制。
class NoCopyByteArrayOutputStream(size: Int) : ByteArrayOutputStream(size) {

    fun fetchBytes(): ByteArray = buf

}