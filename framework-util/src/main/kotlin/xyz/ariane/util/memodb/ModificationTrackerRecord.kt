package xyz.ariane.util.memodb

import com.google.common.hash.HashCode

/**
 * 变更跟踪数据
 */
class ModificationTrackerRecord {

    /**
     * 最后一次[EntityWrapper.hashCode]的值
     * 这是粗检
     */
    var lastIntHashCode: Int = 0

    /**
     * 最后一次序列化字节的hash code
     * 这是细检
     */
    var lastSerBytesHashCode: HashCode? = null

    /** 连续检查[lastIntHashCode]并且无变化的次数 */
    var intHashCodeEqualTimes: Short = 0

    var dirty: Boolean = false

}