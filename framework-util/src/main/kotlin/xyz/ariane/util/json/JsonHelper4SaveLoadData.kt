package xyz.ariane.util.json

import com.fasterxml.jackson.core.JsonProcessingException
import com.fasterxml.jackson.databind.JsonMappingException
import java.io.IOException
import java.nio.ByteBuffer

/**
 * 这个类只能用于在稳定的线程池环境下序列化和反序列化压缩后的数据。
 * 通常这些数据是存储玩家数据用的，而不是进程间通讯的数据。
 */
class JsonHelper4SaveLoadData(val defaultSize: Int = 256 * 1024) {

    // 序列化用内存数组对象池
    // 借助线程本地存储创建和维护缓存对象，以减少不必要的内存分配。
    private val bytes4Json: ThreadLocal<NoCopyByteArrayOutputStream> =
        object : ThreadLocal<NoCopyByteArrayOutputStream>() {

            override fun initialValue(): NoCopyByteArrayOutputStream {
                // 创建一个中间缓存对象
                val bytes = NoCopyByteArrayOutputStream(defaultSize)
                return bytes
            }

        }

    /**
     * 序列化成字节数组
     */
    fun <T> toBytesJson4Big(t: T): ByteBuffer {
        // 序列化
        val bb = bytes4Json.get()

        // 重置！很关键
        bb.reset()

        try {
            mapperUsedInGame.writeValue(bb, t)
        } catch (e: JsonProcessingException) { // to support [JACKSON-758]
            throw e
        } catch (e: IOException) { // shouldn't really happen, but is declared as possibility so:
            throw JsonMappingException.fromUnexpectedIOE(e)
        }
        val result = bb.fetchBytes()
        val size = bb.size()

        // 转成byteBuffer
        val bf = ByteBuffer.wrap(result, 0, size)

        return bf
    }

}