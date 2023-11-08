@file:Suppress("unused")

package xyz.ariane.util.lang

import java.io.IOException

object HexStringUtil {

    private val BYTE2HEX_PAD = arrayOfNulls<String>(256)
    private val BYTE2HEX_NOPAD = arrayOfNulls<String>(256)

    init {
        // Generate the lookup table that converts a byte into a 2-digit hexadecimal integer.
        var i = 0
        while (i < 10) {
            BYTE2HEX_PAD[i] = "0$i"
            BYTE2HEX_NOPAD[i] = i.toString()
            i++
        }
        while (i < 16) {
            val c = ('a' + i - 10)
            BYTE2HEX_PAD[i] = "0$c"
            BYTE2HEX_NOPAD[i] = c.toString()
            i++
        }
        while (i < BYTE2HEX_PAD.size) {
            val str = Integer.toHexString(i)
            BYTE2HEX_PAD[i] = str
            BYTE2HEX_NOPAD[i] = str
            i++
        }
    }

    /**
     * Converts the specified byte value into a 2-digit hexadecimal integer.
     */
    private fun byteToHexStringPadded(value: Int): String = BYTE2HEX_PAD[value and 0xff] as String

    /**
     * Converts the specified byte value into a 2-digit hexadecimal integer and appends it to the specified buffer.
     */
    private fun <T : Appendable> byteToHexStringPadded(buf: T, value: Int): T {
        try {
            buf.append(byteToHexStringPadded(value))
        } catch (e: IOException) {
            throw RuntimeException(e)
        }

        return buf
    }

    /**
     * Converts the specified byte array into a hexadecimal value.
     */
    @JvmOverloads
    fun toHexStringPadded(src: ByteArray, offset: Int = 0, length: Int = src.size): String {
        return toHexStringPadded(StringBuilder(length shl 1), src, offset, length).toString()
    }

    /**
     * Converts the specified byte array into a hexadecimal value and appends it to the specified buffer.
     */
    fun <T : Appendable> toHexStringPadded(dst: T, src: ByteArray): T {
        return toHexStringPadded(dst, src, 0, src.size)
    }

    /**
     * Converts the specified byte array into a hexadecimal value and appends it to the specified buffer.
     */
    private fun <T : Appendable> toHexStringPadded(dst: T, src: ByteArray, offset: Int, length: Int): T {
        val end = offset + length
        for (i in offset until end) {
            byteToHexStringPadded(dst, src[i].toInt())
        }
        return dst
    }

    /**
     * Converts the specified byte value into a hexadecimal integer.
     */
    private fun byteToHexString(value: Int): String = BYTE2HEX_NOPAD[value and 0xff] as String

    /**
     * Converts the specified byte value into a hexadecimal integer and appends it to the specified buffer.
     */
    private fun <T : Appendable> byteToHexString(buf: T, value: Int): T {
        try {
            buf.append(byteToHexString(value))
        } catch (e: IOException) {
            throw RuntimeException(e)
        }

        return buf
    }

    /**
     * Converts the specified byte array into a hexadecimal value.
     */
    @JvmOverloads
    fun toHexString(src: ByteArray, offset: Int = 0, length: Int = src.size): String {
        return toHexString(StringBuilder(length shl 1), src, offset, length).toString()
    }

    /**
     * Converts the specified byte array into a hexadecimal value and appends it to the specified buffer.
     */
    fun <T : Appendable> toHexString(dst: T, src: ByteArray): T {
        return toHexString(dst, src, 0, src.size)
    }

    /**
     * Converts the specified byte array into a hexadecimal value and appends it to the specified buffer.
     */
    private fun <T : Appendable> toHexString(dst: T, src: ByteArray, offset: Int, length: Int): T {
        assert(length >= 0)
        if (length == 0) {
            return dst
        }

        val end = offset + length
        val endMinusOne = end - 1
        var i: Int = offset

        // Skip preceding zeroes.
        while (i < endMinusOne) {
            if (src[i].toInt() != 0) {
                break
            }
            i++
        }

        byteToHexString(dst, src[i++].toInt())
        val remaining = end - i
        toHexStringPadded(dst, src, i, remaining)

        return dst
    }
}
