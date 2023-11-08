package xyz.ariane.util.realm

import com.google.common.base.Preconditions
import com.google.common.collect.Lists
import com.google.common.hash.Hashing
import xyz.ariane.util.lang.isNullOrBlank

import java.io.UnsupportedEncodingException
import java.net.URLDecoder

object PassportUtil {

    /**
     * 验证是否通过
     */
    fun verifyChecksum(checksumSrc: String, verify: String, key: String): Boolean {
        return if (isNullOrBlank(checksumSrc) || isNullOrBlank(verify)) {
            false
        } else {
            builderVerify(checksumSrc, key) == verify
        }
    }

    /**
     * 产生验证信息
     */
    fun builderVerify(checksumSrc: String, key: String): String {
        return Hashing.md5().hashString(checksumSrc + key, Charsets.UTF_8).toString()
    }

    /**
     * 官网checkSum
     */
    fun buildPoint18PlatformCheckSumSrc(paramMap: Map<String, String>): String {
        val list = Lists.newArrayListWithCapacity<String>(paramMap.size)
        for ((name, value) in paramMap) {
            addUTF8String(list, name, value, true)
        }
        return buildCheckSumSrc(list)
    }

    /**
     * Supersdk checkSum
     */
    fun buildSuperSdkCheckSumSrc(paramMap: Map<String, Any>): String {
        val list = Lists.newArrayListWithCapacity<String>(paramMap.size)
        for ((name, v) in paramMap) {
            val valueStr = extractValueString(v)
            val value = try {
                URLDecoder.decode(valueStr, Charsets.UTF_8.name())
            } catch (e: UnsupportedEncodingException) {
                throw RuntimeException(e)
            }

            addUTF8String(list, name, value, true)
        }
        return buildCheckSumSrc(list)
    }

    /**
     * 直接拼接client请求，无需中文解码
     */
    fun buildClientCheckSumSrc(paramMap: Map<String, Any>): String {
        val list = Lists.newArrayListWithCapacity<String>(paramMap.size)
        for ((name, value) in paramMap) {
            val valueString = extractValueString(value)
            if (!isNullOrBlank(valueString)) {
                list.add("$name=$valueString")
            }
        }
        return buildCheckSumSrc(list)
    }

    /**
     * build a string which is used to calculate md5 checksum
     *
     * @param paramMap params
     * @return request string only include fields needed by md5 checksum, sorted
     * lexicographically.
     */
    fun buildChecksumSrc(paramMap: Map<String, Any>): String {
        Preconditions.checkNotNull(paramMap)
        val list = Lists.newArrayListWithCapacity<String>(paramMap.size)
        for ((name, value) in paramMap) {
            val atoken = "tocken"
            if (atoken == name) {
                addUTF8String(list, name, value, false)
            } else if (!isNullOrBlank(name)
                    && "action" != name
                    && "sign" != name
                    && "verify" != name
                    && "token" != name
                    && atoken != name) {
                addUTF8String(list, name, value, false)
            }
        }
        return buildCheckSumSrc(list)
    }

    private fun buildCheckSumSrc(list: List<String>): String {
        list.sortedBy { it }
        val key = StringBuilder()
        for (i in list.indices) {
            key.append(list[i])
            if (i < list.size - 1) {
                key.append("&")
            }
        }
        return key.toString()
    }

    /**
     * @param isAllowNullValue value是否可以为NULL
     */
    private fun addUTF8String(list: MutableList<String>, name: String, v: Any, isAllowNullValue: Boolean) {
        var value = v
        val valueString = extractValueString(value)
        try {
            value = String(valueString.toByteArray(charset("iso8859-1")), Charsets.UTF_8)
        } catch (e: UnsupportedEncodingException) {
            throw RuntimeException(e)
        }

        if (isAllowNullValue || !isNullOrBlank(valueString)) {
            list.add("$name=$value")
        }
    }

    private fun extractValueString(value: Any): String {
        val valueString: String
        if (value is String) {
            valueString = value
        } else if (value is Array<*>) {
            val v0 = value[0]
            if (v0 is String) {
                valueString = v0
            } else {
                throw IllegalArgumentException("unknown param value type: " + value.javaClass.name)
            }
        } else if (value is Number) {
            valueString = value.toString()
        } else {
            throw IllegalArgumentException("unknown param value type: " + value.javaClass.name)
        }
        return valueString
    }

}
