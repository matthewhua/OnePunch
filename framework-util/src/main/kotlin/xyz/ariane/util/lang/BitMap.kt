package xyz.ariane.util.lang

/**
 * 位图简易实现
 */
open class BitMap(
    private val elementData: LongArray
) {

    constructor(max: Int) : this(LongArray(max + 64 shr 6))

    val size: Int
        get() = this.elementData.size * (1 shl 6) - 1

    val rawArray: LongArray
        get() = this.elementData

    /**
     * 往位图里面添加元素
     *
     * @param value 待添加的元素
     */
    operator fun plusAssign(value: Int) {
        elementData[value shr 6] = elementData[value shr 6] or (1L shl (value and 63))
    }

    /**
     * 删除位图里面的元素
     *
     * @param value 待删除的元素
     */
    operator fun minusAssign(value: Int) {
        elementData[value shr 6] = elementData[value shr 6] and (1L shl (value and 63)).inv()
    }

    /**
     * 观察目标元素num在位图中是存在
     *
     * @param value 目标元素
     * @return 存在返回1，否则返回0
     */
    operator fun contains(value: Int): Boolean {
        return elementData[value shr 6] and (1L shl (value and 63)) != 0L
    }

    override fun toString(): String {
        val sb = StringBuilder()
        for (i in elementData.indices) {
            val binaryString = java.lang.Long.toBinaryString(elementData[i])
            sb.append("[$i]:").append(binaryString)
            sb.append("\n")
        }
        return sb.toString()
    }
}

private const val CONST_LONG_BIT = (1 shl 6)

// 预留 [0,64)
class BitMap64 : BitMap(CONST_LONG_BIT - 1 + CONST_LONG_BIT * 0)

// 预留 [0,128)
class BitMap128 : BitMap(CONST_LONG_BIT - 1 + CONST_LONG_BIT * 1)

// 预留 [0,192)
class BitMap192 : BitMap(CONST_LONG_BIT - 1 + CONST_LONG_BIT * 2)

// 预留 [0,256)
class BitMap256 : BitMap(CONST_LONG_BIT - 1 + CONST_LONG_BIT * 3)

// 预留 [0,320)
class BitMap320 : BitMap(CONST_LONG_BIT - 1 + CONST_LONG_BIT * 4)

fun LongArray.toBitMap(): BitMap = BitMap(this)