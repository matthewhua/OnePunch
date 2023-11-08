package xyz.ariane.util.compress

import net.jpountz.lz4.LZ4CompressorWithLength
import net.jpountz.lz4.LZ4DecompressorWithLength
import net.jpountz.lz4.LZ4Factory
import xyz.ariane.util.annotation.AllOpen
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.nio.ByteBuffer
import java.util.zip.GZIPInputStream
import java.util.zip.GZIPOutputStream
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream


var compressUtil = CompressUtil()

class ByteBufferContainer(
    var size: Int,
    var bf: ByteBuffer
) {
    fun ensureSize(needSize: Int) {
        if (size >= needSize) {
            return
        }
        for (i in 1..10) {
            size *= 2
            if (size >= needSize) {
                break
            }
        }

        bf = ByteBuffer.allocate(size)
    }
}

class ByteArrayUncompressContainer(
    var size: Int,
    var ba: ByteArray,
    var dataLength: Int,
    var debugMark: Boolean
) {
    fun ensureSize(needSize: Int) {
        if (size >= needSize) {
            return
        }
        for (i in 1..10) {
            size *= 2
            if (size >= needSize) {
                break
            }
        }

        ba = ByteArray(size)
    }
}

@AllOpen
class CompressUtil(val defaultSize: Int = 128 * 1024) {

    val factory: LZ4Factory = LZ4Factory.fastestInstance()

    // 压缩用内存数组对象池
    // 借助线程本地存储创建和维护缓存对象，以减少不必要的内存分配。
    private val bytes4Zip: ThreadLocal<ByteBufferContainer> =
        object : ThreadLocal<ByteBufferContainer>() {

            override fun initialValue(): ByteBufferContainer {
                // 创建一个中间缓存对象
                val bytes = ByteBuffer.allocate(defaultSize)
                return ByteBufferContainer(defaultSize, bytes)
            }
        }

    // 解压缩用的对象池
    // 借助线程本地存储创建和维护缓存对象，以减少不必要的内存分配。
    private val bytes4Uncompress: ThreadLocal<ByteArrayUncompressContainer> =
        object : ThreadLocal<ByteArrayUncompressContainer>() {

            override fun initialValue(): ByteArrayUncompressContainer {
                // 创建一个中间缓存对象
                val bytes = ByteArray(defaultSize)
                return ByteArrayUncompressContainer(defaultSize, bytes, 0, false)
            }
        }

    /**
     * lz4压缩，前4位带了原始长度
     */
    fun lz4Compress(uncompressData: ByteArray): ByteArray {
        val compressor = factory.fastCompressor()

        val lengthCompressor = LZ4CompressorWithLength(compressor)

        val compressData = lengthCompressor.compress(uncompressData)

        return compressData
    }

    /**
     * lz4压缩，前4位带了原始长度
     */
    fun lz4Compress(uncompressByteBuffer: ByteBuffer): ByteArray {
        val compressor = factory.fastCompressor()

        val lengthCompressor = LZ4CompressorWithLength(compressor)

        val bfContainer = bytes4Zip.get()

        val maxLen = lengthCompressor.maxCompressedLength(uncompressByteBuffer.limit())
        bfContainer.ensureSize(maxLen)

        val destByteBuffer = bfContainer.bf

        // 重置！很关键
        destByteBuffer.clear()

        lengthCompressor.compress(uncompressByteBuffer, destByteBuffer)

        // 反转一下，未使用的变以使用，之后取剩余的
        destByteBuffer.flip()

        val uncompressBytes = ByteArray(destByteBuffer.remaining())
        destByteBuffer.get(uncompressBytes)

        return uncompressBytes
    }

    /**
     * lz4压缩，前4位带原始长度
     */
    fun lz4Uncompress(compressData: ByteArray): Pair<ByteArray?, Exception?> {
        val decompressor = factory.fastDecompressor()

        val lengthDecompressor = LZ4DecompressorWithLength(decompressor)

        try {
            val decompressData = lengthDecompressor.decompress(compressData)

            return Pair(decompressData, null)

        } catch (ex: Exception) {
            return Pair(null, ex)
        }
    }

    /**
     * lz4压缩，前4位带原始长度
     */
    fun lz4UncompressWithBuffer(compressData: ByteArray): Pair<ByteArrayUncompressContainer?, Exception?> {
        val decompressor = factory.fastDecompressor()

        val lengthDecompressor = LZ4DecompressorWithLength(decompressor)

        try {
            // 得到解压缩数据的长度
            val len = LZ4DecompressorWithLength.getDecompressedLength(compressData)

            val bfContainer = bytes4Uncompress.get()
//            bfContainer.debugMark = true // 调试用标记
            bfContainer.ensureSize(len) // 保险做法，加一点余量

            // 解压缩
            val destByteBuffer = bfContainer.ba
            lengthDecompressor.decompress(compressData, destByteBuffer)
            bfContainer.dataLength = len

            return Pair(bfContainer, null)

        } catch (ex: Exception) {
            return Pair(null, ex)
        }
    }

    /***
     * 压缩GZip
     */
    fun gzipCompress(data: ByteArray): ByteArray {
        val bos = ByteArrayOutputStream()
        val gzip = GZIPOutputStream(bos)
        gzip.write(data)
        gzip.finish()
        gzip.close()
        val b = bos.toByteArray()
        bos.close()
        return b
    }

    /***
     * 解压GZip
     */
    fun gzipUncompress(data: ByteArray): Pair<ByteArray?, Exception?> {
        var b: ByteArray? = null
        try {
            val bis = ByteArrayInputStream(data)
            val gzip = GZIPInputStream(bis)
            val buf = ByteArray(1024)
            val baos = ByteArrayOutputStream()
            while (true) {
                val num = gzip.read(buf, 0, buf.size)
                if (num == -1) {
                    break
                }
                baos.write(buf, 0, num)
            }
            b = baos.toByteArray()
            baos.flush()
            baos.close()
            gzip.close()
            bis.close()

            return Pair(b, null)
        } catch (ex: Exception) {
            return Pair(null, ex)
        }
    }

    /**
     * zip文件流内存解压
     */
    fun zipUncompress(inputStream: InputStream): Map<String, ByteArray> {
        val fileMap = hashMapOf<String, ByteArray>()

        ZipInputStream(inputStream).use { zipStream ->
            var ze: ZipEntry? = zipStream.nextEntry

            while (ze != null) {
                if (!ze.isDirectory) {
                    //截取文件名
                    val fileName = ze.name.substring(ze.name.lastIndexOf("/") + 1)
                    val byteArrayOutputStream = ByteArrayOutputStream()
                    val buffer = ByteArray(1024)
                    var length = zipStream.read(buffer, 0, buffer.size)
                    while (length > -1) {
                        byteArrayOutputStream.write(buffer, 0, length)
                        length = zipStream.read(buffer, 0, buffer.size)
                    }
                    fileMap[fileName] = byteArrayOutputStream.toByteArray()
                    byteArrayOutputStream.close()
                }
                ze = zipStream.nextEntry
            }
        }
        return fileMap
    }
}
