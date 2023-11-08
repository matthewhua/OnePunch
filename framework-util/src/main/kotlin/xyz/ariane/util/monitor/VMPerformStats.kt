package xyz.ariane.util.monitor

import com.sun.management.OperatingSystemMXBean
import java.io.Serializable
import java.lang.management.ManagementFactory

data class GcStat(
    var gcName: String = "", // gc名
    var gcCount: Long = 0, // gc次数
    var gcTime: Long = 0 // gc耗时
)

/**
 * jvm性能统计数据
 */
class VMPerformStats : Serializable {
    /** JVM使用的全部内存(bytes) */
    var totalJvmMemory: Long = 0
        private set

    /** JVM剩余内存(bytes) */
    var freeJvmMemory: Long = 0
        private set

    /** OS全部物理内存(bytes) */
    var totalPhysicalMemory: Long = 0
        private set

    /** OS剩余物理内存(bytes) */
    var freePhysicalMemory: Long = 0
        private set

    /** OS平均负载 [OperatingSystemMXBean.getSystemLoadAverage] */
    var systemLoadAverage: Double = 0.toDouble()
        private set

    /** cpu核数 */
    var availableProcessors: Int = 0
        private set

    var gcs: MutableList<GcStat> = arrayListOf()

    companion object {

        private const val serialVersionUID = 7946825439446021379L

        /**
         * 生成JVM性能报告
         */
        fun generate(): VMPerformStats {
            val stats = VMPerformStats()
            val mxbean = ManagementFactory.getOperatingSystemMXBean()
            val rt = Runtime.getRuntime()
            stats.totalJvmMemory = rt.totalMemory()
            stats.freeJvmMemory = rt.freeMemory()
            stats.systemLoadAverage = mxbean.systemLoadAverage
            stats.availableProcessors = mxbean.availableProcessors
            if (mxbean is OperatingSystemMXBean) {
                stats.freePhysicalMemory = mxbean.freePhysicalMemorySize
                stats.totalPhysicalMemory = mxbean.totalPhysicalMemorySize
            }

            val gcMxBeans = ManagementFactory.getGarbageCollectorMXBeans()
            for (gcBean in gcMxBeans) {
                val gcStat = GcStat(gcBean.name, gcBean.collectionCount, gcBean.collectionTime)
                stats.gcs.add(gcStat)
            }
            return stats
        }
    }
}

