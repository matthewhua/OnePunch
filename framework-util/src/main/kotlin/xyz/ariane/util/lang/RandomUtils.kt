package xyz.ariane.util.lang

import com.google.common.base.Preconditions
import com.google.common.math.IntMath
import xyz.ariane.util.math.Multiplier

import java.util.*
import java.util.concurrent.ThreadLocalRandom

object RandomUtils {

    fun nextInt(i: Int): Int {
        return if (i > 0) {
            ThreadLocalRandom.current().nextInt(i)
        } else {
            0
        }
    }

    /**
     * 随机选择一个
     */
    fun <T> select(lib: Array<T>): T {
        return lib[between(0, lib.size - 1)]
    }

    inline fun <reified T> select(values: Collection<T>): T {
        Preconditions.checkNotNull(values)
        Preconditions.checkArgument(!values.isEmpty(), "Empty collection")
        return select(values.toTypedArray())
    }

    /**
     * 随机选取n个不重复的元素。不改变原来的values的结构和顺序等
     */
    fun <T> select(values: Collection<T>, n: Int): List<T> {
        Preconditions.checkNotNull(values)
        if (values.isEmpty()) {
            return emptyList()
        }
        val list = ArrayList(values)
        if (list.size <= n) {
            return list
        }
        Collections.shuffle(list)
        return list.subList(0, n)
    }

    fun randomBoolean(): Boolean {
        return ThreadLocalRandom.current().nextBoolean()
    }

    /**
     * 获取[min,max]区间的随机整数,包括两端
     */
    fun between(min: Int, max: Int): Int {
        return between(min.toLong(), max.toLong()).toInt()
    }

    fun between(min: Long, max: Long): Long {
        Preconditions.checkArgument(min <= max)
        return if (max - min > 0) {
            ThreadLocalRandom.current().nextLong(max - min + 1) + min
        } else {
            min
        }
    }

    fun between(range: IntRange): Int {
        return between(range.start, range.endInclusive)
    }

    fun between(range: LongRange): Long {
        return between(range.start, range.endInclusive)
    }

    fun selectIndexByFrequency(vararg weights: Int): Int {
        Preconditions.checkArgument(weights.size > 0)
        var total = 0
        for (w in weights) {
            Preconditions.checkArgument(w >= 0, "Invalidate weight %s.", w)
            total = IntMath.checkedAdd(total, w)
        }
        // weights中的权重不能全部为0
        Preconditions.checkArgument(total > 0, "All weights are 0.")
        val r = between(1, total)
        var sum = 0
        for (i in weights.indices) {
            sum += weights[i]
            if (r <= sum) {
                return i
            }
        }
        return 0
    }

    interface HasFrequency {

        val frequency: Int
    }

    /**
     * 按频率分布选择一个
     *
     * @param dist 频率分布
     * @return 选中的元素，如果dist为空或频率都为0，返回null
     */
    fun <T : HasFrequency> selectByFrequency(dist: Collection<T>): T? {
        if (dist.isEmpty()) {
            return null
        }
        var total = 0
        for (t in dist) {
            val freq = t.frequency
            Preconditions.checkArgument(freq >= 0, "Invalidate frequency %s", freq)
            total = IntMath.checkedAdd(total, freq)
        }
        if (total > 0) {
            val r = between(1, total)
            var sum = 0
            for (t in dist) {
                sum += t.frequency
                if (r <= sum) {
                    return t
                }
            }
        }
        return null
    }

    /**
     * 按频率分布选择num个，如果dist集合中频率大于0元素数量小于num，那么最终结果只会返回频率大于0的记录。
     *
     * @param dist 频率分布
     * @param num  选取个数
     * @return 选中的元素列表
     */
    fun <T : HasFrequency> selectByFrequency(dist: Collection<T>, num: Int): List<T> {
        Preconditions.checkArgument(num > 0, "num必须>0")

        val candidates = LinkedList(dist)
        val result = ArrayList<T>(num)
        var i = 0
        while (i < num && !candidates.isEmpty()) {
            val selected = selectByFrequency(candidates)
            if (selected != null) {
                result.add(selected)
                candidates.remove(selected)
            }
            i++
        }
        return result
    }

    /**
     * 随机发生
     *
     * @param probability 整数表示的概率,实际概率为probability/base
     * @param base        基数,百分比为100,千分比为1000
     * @return 是否发生
     */
    fun drawHappen(probability: Int, base: Int): Boolean {
        if (probability >= base) {
            return true
        } else if (probability <= 0) {
            return false
        } else {
            val randInt = between(1, base)
            return randInt <= probability
        }
    }

    /**
     * 按对称的比例随机浮动一个整数
     *
     * @param value     原整数
     * @param floatRate 浮动率
     * @param base      浮动率分母
     * @return 随机浮动后的整数
     */
    fun symmetryFloatInt(value: Int, floatRate: Int, base: Int): Int {
        val randomRate = between(-floatRate, floatRate)
        return value * (base + randomRate) / base
    }

    fun symmetryFloatInt(mul: Multiplier, floatRate: Int, base: Int) {
        val randomRate = between(-floatRate, floatRate)
        mul.multiOnePlus(randomRate.toLong(), base.toLong())
    }

}
