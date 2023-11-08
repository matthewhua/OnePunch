package xyz.ariane.util.lang

import java.util.*

fun isWindows(): Boolean {
    val osName = System.getProperty("os.name")
    return osName.lowercase(Locale.getDefault()).contains("windows")
}

fun isLinux(): Boolean {
    val osName = System.getProperty("os.name")
    return osName.lowercase(Locale.getDefault()).contains("linux")
}

