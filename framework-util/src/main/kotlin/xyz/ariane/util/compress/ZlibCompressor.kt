package xyz.ariane.util.compress

import java.util.zip.Deflater

/**
 * 用于压缩
 */
interface Compressor {

    fun compress(bytes: ByteArray): Pair<Int, ByteArray>
}

class ZlibCompressor(compressLevel: Int) : Compressor {

    private val deflater: Deflater = Deflater(compressLevel)

    override fun compress(bytes: ByteArray): Pair<Int, ByteArray> {
        deflater.reset()
        deflater.setInput(bytes)
        deflater.finish()
        val bytes2 = ByteArray(bytes.size + 50)
        val len: Int = deflater.deflate(bytes2)
        return len to bytes2
    }
}