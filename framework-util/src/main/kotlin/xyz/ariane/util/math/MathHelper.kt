package xyz.ariane.util.math

import xyz.ariane.util.annotation.AllOpen
import xyz.ariane.util.lang.doubleIntAndLongConvert
import java.io.Serializable
import java.util.*
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sqrt

data class Vector2(val x: Int, val y: Int) : Serializable

data class LineSegment(val p1: Vector2, val p2: Vector2)

data class Rectangle(val topLeft: Vector2, val bottomRight: Vector2)

var mathHelper = MathHelper()

@AllOpen
class MathHelper {
    fun calDistance(p1: Vector2, p2: Vector2): Double {
        val xDis = p1.x - p2.x
        val yDis = p1.y - p2.y
        return sqrt((xDis * xDis + yDis * yDis).toDouble())
    }

    /**
     * 检测点是否在三角形内
     */
    fun checkInTriangle(p: Vector2, a: Vector2, b: Vector2, c: Vector2): Boolean {
        var va = 0
        var vb = 0
        var vc = 0

        val ma = Vector2(p.x - a.x, p.y - a.y)
        val mb = Vector2(p.x - b.x, p.y - b.y)
        val mc = Vector2(p.x - c.x, p.y - c.y)

        /*向量叉乘*/
        va = ma.x * mb.y - ma.y * mb.x
        vb = mb.x * mc.y - mb.y * mc.x
        vc = mc.x * ma.y - mc.y * ma.x

        return (va <= 0 && vb <= 0 && vc <= 0) || (va > 0 && vb > 0 && vc > 0)
    }

    /**
     * 获取三角形内所有点(三角形栅格化)
     * Bresenham算法
     * 参考链接
     * http://www.sunshine2k.de/coding/java/TriangleRasterization/TriangleRasterization.html
     * https://www.cnblogs.com/ourroad/archive/2013/05/14/3078841.html
     */
    fun findAllPointInTriangle(a: Vector2, b: Vector2, c: Vector2): List<Vector2> {
        // 首先通过y坐标升序对三个顶点进行排序，因此v1是最高顶点
        val vList = listOf(a, b, c).sortedBy { doubleIntAndLongConvert.doubleInt2Long(it.y, it.x) }

        //在这里我们知道v1.y <= v2.y <= v3.y
        val v1 = vList[0]
        val v2 = vList[1]
        val v3 = vList[2]

        val posList = LinkedList<Vector2>()

        if (v2.y == v3.y) {
            //平底三角
            fillFlatTriangle(posList, v1, v2, v3)
        } else if (v1.y == v2.y) {
            //平顶三角
            fillFlatTriangle(posList, v3, v1, v2)
        } else {
            //将三角形分成一个顶部平坦和一个底部平坦
            val v4 = Vector2(v1.x + ((v2.y - v1.y).toFloat() / (v3.y - v1.y) * (v3.x - v1.x)).toInt(), v2.y)
            fillFlatTriangle(posList, v1, v2, v4)
            fillFlatTriangle(posList, v3, v2, v4)
        }

        return posList
    }

    /**
     * 填充平底/平顶三角形
     * v2.y == v3.y
     */
    fun fillFlatTriangle(posList: LinkedList<Vector2>, v1: Vector2, v2: Vector2, v3: Vector2) {
        if (v2.y != v3.y) {
            return
        }

        var minV = v2
        var maxV = v3
        if (v2.x > v3.x) {
            minV = v3
            maxV = v2
        }

        posList.add(v1)

        var left = v1
        var right = v1
        for (y in min(v1.y, minV.y)..max(v1.y, minV.y)) {
            val newleft = stepLineGrid(left, minV)
            if (newleft == null) {
                break
            }
            val newRight = stepLineGrid(right, maxV)
            if (newRight == null) {
                break
            }
            left = newleft
            right = newRight

            for (x in left.x..right.x) {
                posList.add(Vector2(x, left.y))
            }
        }
    }

    /**
     * Bresenham算法栅格化，y方向步进
     */
    fun stepLineGrid(v1: Vector2, v2: Vector2): Vector2? {

        val dx = abs(v1.x - v2.x)
        val dy = abs(v1.y - v2.y)
        var x = v1.x
        var y = v1.y
        val sx = if (v2.x > v1.x) 1 else -1
        val sy = if (v2.y > v1.y) 1 else -1

        if (dx > dy) {
            var e = -dx
            for (i in 0 until dx) {
                x += sx
                e += 2 * dy
                if (e >= 0) {
                    y += sy
                    e -= 2 * dx

                    return Vector2(x, y)
                }
            }
        } else {
            var e = -dy
            for (i in 0 until dy) {
                y += sy
                e += 2 * dx
                if (e >= 0) {
                    x += sx
                    e -= 2 * dy
                }
                return Vector2(x, y)
            }
        }

        return null
    }

    /**
     * 线段栅格化(Bresenham算法)
     */
    fun lineGrid(v1: Vector2, v2: Vector2): List<Vector2> {
        val posList = LinkedList<Vector2>()
        posList.add(v1)

        val dx = abs(v1.x - v2.x)
        val dy = abs(v1.y - v2.y)
        var x = v1.x
        var y = v1.y
        val sx = if (v2.x > v1.x) 1 else -1
        val sy = if (v2.y > v1.y) 1 else -1

        if (dx > dy) {
            var e = -dx
            for (i in 0 until dx) {
                x += sx
                e += 2 * dy
                if (e >= 0) {
                    y += sy
                    e -= 2 * dx
                }
                posList.add(Vector2(x, y))
            }
        } else {
            var e = -dy
            for (i in 0 until dy) {
                y += sy
                e += 2 * dx
                if (e >= 0) {
                    x += sx
                    e -= 2 * dy
                }
                posList.add(Vector2(x, y))
            }
        }
        return posList
    }

    //要按照螺旋顺序
    val Right = 0
    val Down = 1
    val Left = 2
    val Up = 3

    /**
     * 螺旋矩阵
     * 思路是选定一个点作为中心点，向外辐射，如果点在指定格点内，则加入队列，直至全部格点都落入格点
     */
    fun helixArray(x: Int, y: Int, checkOk: (posX: Int, posY: Int) -> Boolean): Vector2? {
        var posX = x
        var posY = y

        var dir = Right
        var len = 1
        var count = 0
        for (round in 0..1000) {
            for (i in 0 until len) {
                when (dir) {
                    Left -> posX--
                    Right -> posX++
                    Up -> posY--
                    Down -> posY++
                }

                if (posX < 0 || posY < 0) {
                    continue
                }

                if (checkOk.invoke(posX, posY)) {
                    return Vector2(posX, posY)
                }
            }
            count++
            if (count == 2) {
                count = 0
                len++
            }
            dir = (dir + 1) % 4
        }

        return null
    }

    fun helixArray(x: Long, y: Long, checkOk: (posX: Long, posY: Long) -> Boolean) {
        var posX = x
        var posY = y

        var dir = Right
        var len = 1
        var count = 0
        for (round in 0..1000) {
            for (i in 0 until len) {
                when (dir) {
                    Left -> posX--
                    Right -> posX++
                    Up -> posY--
                    Down -> posY++
                }

                if (posX < 0 || posY < 0) {
                    continue
                }

                if (checkOk.invoke(posX, posY)) {
                    return
                }
            }
            count++
            if (count == 2) {
                count = 0
                len++
            }
            dir = (dir + 1) % 4
        }
    }

    /**
     * 由外向内的螺旋矩阵
     */
    fun innerHelixArray(width: Int, height: Int, checkOk: (posX: Int, posY: Int) -> Boolean): Vector2? {
        if (width <= 0 || height <= 0) {
            return null
        }
        var minW = 0
        var maxW = width - 1
        var minH = 0
        var maxH = height - 1

        var dir = Right

        for (round in 1..width * height) {
            if (minW > maxW || minH > maxH) {
                break
            }
            when (dir) {
                Right -> {
                    for (w in minW..maxW) {
                        val pos = Vector2(w, minH)
                        if (checkOk(pos.x, pos.y)) {
                            return pos
                        }
                    }
                    minH++
                }
                Down -> {
                    for (h in minH..maxH) {
                        val pos = Vector2(maxW, h)
                        if (checkOk(pos.x, pos.y)) {
                            return pos
                        }
                    }
                    maxW--
                }
                Left -> {
                    for (w in maxW downTo minW) {
                        val pos = Vector2(w, maxH)
                        if (checkOk(pos.x, pos.y)) {
                            return pos
                        }
                    }
                    maxH--
                }
                Up -> {
                    for (h in maxH downTo minH) {
                        val pos = Vector2(minW, h)
                        if (checkOk(pos.x, pos.y)) {
                            return pos
                        }
                    }
                    minW++
                }
            }

            dir = (dir + 1) % 4
        }
        return null
    }

    /**
     * 由外向内的螺旋矩阵
     */
    fun createInnerHelixArray(width: Int, height: Int): List<Vector2> {
        val posArray = arrayListOf<Vector2>()

        var minW = 0
        var maxW = width - 1
        var minH = 0
        var maxH = height - 1

        var dir = Right

        for (round in 1..width * height) {
            if (minW > maxW || minH > maxH) {
                break
            }
            when (dir) {
                Right -> {
                    for (w in minW..maxW) {
                        posArray.add(Vector2(w, minH))
                    }
                    minH++
                }
                Down -> {
                    for (h in minH..maxH) {
                        posArray.add(Vector2(maxW, h))
                    }
                    maxW--
                }
                Left -> {
                    for (w in maxW downTo minW) {
                        posArray.add(Vector2(w, maxH))
                    }
                    maxH--
                }
                Up -> {
                    for (h in maxH downTo minH) {
                        posArray.add(Vector2(minW, h))
                    }
                    minW++
                }
            }

            dir = (dir + 1) % 4
        }

        return posArray
    }

    /**
     * 判断线段与矩形是否相交
     *
     * @param lineSegment LineSegment 线段
     * @param rectangle Rectangle 平行坐标轴的矩形
     * @return Boolean 是否相交
     */
    fun doLineSegmentAndRectangleIntersect(lineSegment: LineSegment, rectangle: Rectangle): Boolean {
        val minX = minOf(rectangle.topLeft.x, rectangle.bottomRight.x)
        val maxX = maxOf(rectangle.topLeft.x, rectangle.bottomRight.x)
        val minY = minOf(rectangle.topLeft.y, rectangle.bottomRight.y)
        val maxY = maxOf(rectangle.topLeft.y, rectangle.bottomRight.y)

        val lineMinX = minOf(lineSegment.p1.x, lineSegment.p2.x)
        val lineMaxX = maxOf(lineSegment.p1.x, lineSegment.p2.x)
        val lineMinY = minOf(lineSegment.p1.y, lineSegment.p2.y)
        val lineMaxY = maxOf(lineSegment.p1.y, lineSegment.p2.y)

        if (lineMaxX < minX || lineMinX > maxX || lineMaxY < minY || lineMinY > maxY) {
            return false
        }

        return true
    }
}

