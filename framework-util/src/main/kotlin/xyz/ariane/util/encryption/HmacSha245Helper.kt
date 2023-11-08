package xyz.ariane.util.encryption

import xyz.ariane.util.annotation.AllOpen
import java.util.*
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

var hmacSha245Helper = HmacSha245Helper()

@AllOpen
class HmacSha245Helper {
    fun sha256HMAC(message: String, secret: String): String {
        var hash = ""
        try {
            val sha256HMAC = Mac.getInstance("HmacSHA256")
            val secretKey = SecretKeySpec(secret.toByteArray(), "HmacSHA256")
            sha256HMAC.init(secretKey)
            val bytes = sha256HMAC.doFinal(message.toByteArray())
            hash = byteArrayToHexString(bytes)
        } catch (e: Exception) {
            println("Error HmacSHA256 ===========" + e.message)
        }

        return hash
    }

    /**
     * 将加密后的字节数组转换成字符串
     *
     * @param b 字节数组
     * @return 字符串
     */
    fun byteArrayToHexString(b: ByteArray?): String {
        val hs = StringBuilder()
        var stmp: String
        var n = 0
        while (b != null && n < b.size) {
            stmp = Integer.toHexString((b[n]).toInt() and 0XFF)
            if (stmp.length == 1)
                hs.append('0')
            hs.append(stmp)
            n++
        }
        return hs.toString().lowercase(Locale.getDefault())
    }
}