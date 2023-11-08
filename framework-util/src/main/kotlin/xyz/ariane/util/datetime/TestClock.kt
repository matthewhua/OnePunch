package xyz.ariane.util.datetime

import java.time.Clock
import java.time.Instant
import java.time.ZoneId

/**
 * 用于单元测试
 */
class TestClock @JvmOverloads constructor(
    @Volatile var delegateClock: Clock = Clock.systemDefaultZone()
) : Clock() {

    override fun getZone(): ZoneId = delegateClock.zone

    override fun withZone(zone: ZoneId?): Clock = delegateClock.withZone(zone)

    override fun instant(): Instant = delegateClock.instant()

}

