package xyz.ariane.util.encryption

import xyz.ariane.util.annotation.AllOpen
import java.nio.charset.Charset
import java.security.MessageDigest

var md5Helper = Md5Helper()

@AllOpen
class Md5Helper {

    fun md5(s: String): String {
        return md5(s.toByteArray())
    }

    fun md5Gbk(s: String): String {
        return md5(s.toByteArray(Charset.forName("GBK")))
    }

    private fun md5(bytes: ByteArray): String {
        val instance = MessageDigest.getInstance("MD5") //获取md5加密对象
        val digest = instance.digest(bytes)   //对字符串见米，返回字节数组
        val sb = StringBuffer()
        for (b in digest) {
            val i = b.toInt() and 0xff  //获取低8位有效值
            var hexString = Integer.toHexString(i) //整数转16进制
            if (hexString.length < 2) {
                hexString = "0$hexString"
            }
            sb.append(hexString)
        }
        return sb.toString()
    }
}