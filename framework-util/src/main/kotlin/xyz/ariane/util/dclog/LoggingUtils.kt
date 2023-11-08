@file:Suppress("unused")

package xyz.ariane.util.dclog

import akka.event.LoggingAdapter
import org.slf4j.Logger

// - slf4j相关的辅助方法。

inline fun Logger.lzDebug(buildMsg: () -> String) {
    if (isDebugEnabled) {
        debug(buildMsg())
    }
}

inline fun Logger.lzInfo(buildMsg: () -> String) {
    if (isInfoEnabled) {
        info(buildMsg())
    }
}

inline fun Logger.lzTrace(buildMsg: () -> String) {
    if (isTraceEnabled) {
        trace(buildMsg())
    }
}

inline fun Logger.lzWarn(buildMsg: () -> String) {
    if (isWarnEnabled) {
        warn(buildMsg())
    }
}

// - akka logger相关的辅助方法。

inline fun LoggingAdapter.lzDebug(buildMsg: () -> String) {
    if (isDebugEnabled) {
        debug(buildMsg())
    }
}

inline fun LoggingAdapter.lzInfo(buildMsg: () -> String) {
    if (isInfoEnabled) {
        info(buildMsg())
    }
}

inline fun LoggingAdapter.lzWarn(buildMsg: () -> String) {
    if (isWarningEnabled) {
        warning(buildMsg())
    }
}

inline fun LoggingAdapter.lzError(buildMsg: () -> String) {
    if (isErrorEnabled) {
        error(buildMsg())
    }
}

