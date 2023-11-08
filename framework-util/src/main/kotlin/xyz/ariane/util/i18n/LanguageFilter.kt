package xyz.ariane.util.i18n

import com.google.common.base.Preconditions.checkNotNull
import com.google.common.collect.Maps
import org.slf4j.LoggerFactory
import xyz.ariane.util.lang.isNullOrBlank
import xyz.ariane.util.lang.notNullOrBlank
import xyz.ariane.util.lang.onlyContainsCn
import java.io.BufferedReader
import java.io.InputStreamReader
import java.nio.charset.Charset
import java.util.*

/**
 * 消息过滤 http://blog.shilimin.com/298.htm
 */
class LanguageFilter @Throws(Exception::class)
constructor(resName: String, private val lang: Language) {

    private val rootMap: MutableMap<Char, Any>

    init {
        notNullOrBlank(resName)
        checkNotNull(lang)
        val res = LanguageFilter::class.java.classLoader.getResource("i18n/$resName")
        checkNotNull(res, "Cannot find %s", resName)

        val conn = res.openConnection()
        conn.useCaches = false
        val inputStream = conn.getInputStream()
        checkNotNull(inputStream, "Cannot find %s", resName)

        rootMap = Maps.newHashMap()
        var wordNum = 0
        BufferedReader(InputStreamReader(inputStream, Charset.forName("utf-8"))).use { reader ->
            var words = reader.readLine()
            while (words != null) {
                try {
                    addWord(words, rootMap)
                    ++wordNum
                } catch (e: Exception) {
                    LOGGER.error("", e)
                }

                words = reader.readLine()
            }
        }
        LOGGER.info("Filter of {} initialized, {} words.", resName, wordNum)
    }

    /**
     * 返回检索到的敏感词
     */
    fun searchSensitiveMessage(source: String): String? {
        if (notNullOrBlank(source)) {
            val points = findSensitiveWords(source)
            if (points.isNotEmpty()) {
                val point = points[0]
                return source.substring(point.start, point.end + 1)
            }
        }
        return null
    }

    fun filterSensitiveMessage(source: String, replacement: Char): String {
        val points = findSensitiveWords(source)
        return replaceWords(source, replacement, points).toString()
    }

    fun filterScript(source: String, replacement: String): String {
        return source
    }

    fun containsSensitiveMessage(source: String): Boolean {
        return searchSensitiveMessage(source) != null
    }

    /**
     * 注意：该方法是将"<>"标签及其中的内容当做整体替换
     */
    fun filterHtml(source: String, replacement: String): String {
        return source.replace(REGXPFORHTML.toRegex(), replacement)
    }

    protected fun findSensitiveWords(source: String): List<Point> {
        if (isNullOrBlank(source)) {
            return listOf()
        }

        val points = ArrayList<Point>()
        var nowMap: Map<Char, Any>? = rootMap
        var start = 0
        for (i in 0 until source.length) {
            val c = source[i]
            if (c == '^') {
                start = i + 1
                nowMap = rootMap
            } else {
                val v1 = if (nowMap == null) null else nowMap[c]
                if (v1 is Map<*, *>) {
                    nowMap = v1 as Map<Char, Any>
                    if ("1" == nowMap['^']) { // 结束
                        points.add(Point(start, i))
                        nowMap = rootMap
                        start = i + 1
                    }
                } else {
                    val v2 = rootMap[c]
                    if (v2 is Map<*, *>) {
                        start = i
                        nowMap = v2 as Map<Char, Any>
                        if ("1" == nowMap['^']) { // 结束
                            points.add(Point(start, i))
                            nowMap = rootMap
                            start = i + 1
                        }
                    } else if (lang === Language.ko_KR) {
                        if (onlyContainsCn(c.toString())) {
                            points.add(Point(start, i))
                            nowMap = rootMap
                            start = i + 1
                        }
                    } else {
                        start = i + 1
                        nowMap = rootMap
                    }
                }
            }
        }
        return points
    }

    protected fun replaceWords(source: String, replacement: Char, points: List<Point>): StringBuilder {
        if (points.size == 0) {
            return StringBuilder(source)
        }

        var start = 0
        val results = StringBuilder()
        for (i in points.indices) {
            val point = points[i]
            results.append(source.substring(start, point.start))
            start = point.end + 1
            val replacements = repeat(point.end - point.start + 1, replacement)
            results.append(replacements)

            if (i == points.size - 1) {
                results.append(source.substring(point.end + 1))
            }
        }
        return results
    }

    /**
     * 屏蔽字字典类型
     */
    enum class Type(val dic: String) {
        words("words.dic"),
        names("name.dic");

        fun genDictFileName(language: Language): String {
            return if (language === Language.zh_CN) {
                dic
            } else {
                language.name + "_" + dic
            }
        }

    }

    class Point(val start: Int, val end: Int)

    companion object {

        val REGXPFORHTML = "<([^>]*)>"

        private val LOGGER = LoggerFactory.getLogger(LanguageFilter::class.java)

        private fun addWord(word: String, map: MutableMap<Char, Any>) {
            var nowMap: MutableMap<Char, Any> = map
            for (j in 0 until word.length) {
                val c = word[j]
                val value = nowMap[c]
                if (value == null) {
                    val newMap = HashMap<Char, Any>()
                    newMap['^'] = "0"
                    nowMap[c] = newMap
                    nowMap = newMap
                } else if (value is MutableMap<*, *>) {
                    nowMap = value as MutableMap<Char, Any>
                }
                if (j == word.length - 1) {
                    nowMap['^'] = "1"
                }
            }
        }

        protected fun repeat(repeat: Int, ch: Char): String {
            val buf = CharArray(repeat)
            for (i in repeat - 1 downTo 0) {
                buf[i] = ch
            }
            return String(buf)
        }
    }
}
