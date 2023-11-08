package xyz.ariane.util.compress

import java.io.*
import java.nio.charset.Charset
import java.util.zip.Deflater
import java.util.zip.DeflaterOutputStream
import java.util.zip.InflaterInputStream
import java.util.zip.InflaterOutputStream

/**
 * 压缩工具
 */
object Zipper {

    /**
     * 解压缩
     */
    fun decompress(compressedData: ByteArray): ByteArray {
        try {
            ByteArrayOutputStream().use { bos ->
                InflaterOutputStream(bos).use { zos ->
                    zos.write(compressedData)
                    zos.finish()
                    return bos.toByteArray()
                }
            }
        } catch (ex: Exception) {
            throw RuntimeException(ex)
        }

    }

    /**
     * 单行压缩字符串解压
     */
    @JvmOverloads
    fun decompressLine(compressedData: ByteArray, charset: Charset = Charsets.UTF_8): String {
        return decompressLine(ByteArrayInputStream(compressedData), charset)
    }

    /**
     * 单行压缩字符串解压
     */
    @JvmOverloads
    fun decompressLine(input: InputStream, charset: Charset = Charsets.UTF_8): String {
        try {
            BufferedReader(InputStreamReader(InflaterInputStream(input), charset))
                    .use { reader -> return reader.readLine() }
        } catch (e: IOException) {
            throw RuntimeException(e)
        }

    }

    /**
     * 压缩
     */
    @JvmOverloads
    fun compress(data: String, charset: Charset = Charsets.UTF_8, level: Int = Deflater.BEST_SPEED): ByteArray {
        try {
            return compress(data.toByteArray(charset), level)
        } catch (e: Exception) {
            throw RuntimeException(e)
        }

    }

    /**
     * 压缩
     */
    @JvmOverloads
    fun compress(data: ByteArray, level: Int = Deflater.BEST_SPEED): ByteArray {
        try {
            ByteArrayOutputStream().use { bos ->
                DeflaterOutputStream(bos, Deflater(level)).use { dzos ->
                    dzos.write(data)
                    dzos.finish()
                    return bos.toByteArray()
                }
            }
        } catch (ex: Exception) {
            throw RuntimeException(ex)
        }

    }

}
