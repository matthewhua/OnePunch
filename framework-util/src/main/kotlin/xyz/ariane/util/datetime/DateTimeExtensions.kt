package xyz.ariane.util.datetime

import java.time.Instant
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.ZonedDateTime
import java.util.*

val systemDefaultZoneId: ZoneId = ZoneId.systemDefault()

fun toDefaultZonedDateTime(epochMilli: Long): ZonedDateTime =
    Instant.ofEpochMilli(epochMilli).atZone(systemDefaultZoneId)

fun Instant.atDefaultZone(): ZonedDateTime = atZone(systemDefaultZoneId)

fun LocalDateTime.atDefaultZone(): ZonedDateTime = atZone(systemDefaultZoneId)

fun LocalDateTime.toDefaultEpochMilli(): Long = atZone(systemDefaultZoneId).toInstant().toEpochMilli() // 转成当前毫秒数

fun Date.toZonedDateTime(): ZonedDateTime = ZonedDateTime.ofInstant(toInstant(), systemDefaultZoneId)

fun ZonedDateTime.toDate(): Date = Date.from(toInstant())