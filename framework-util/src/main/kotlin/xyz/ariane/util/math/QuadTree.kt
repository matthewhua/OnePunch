package xyz.ariane.util.math

import xyz.ariane.util.annotation.AllOpen
import java.util.*


/**
 * 四叉树
 */
@AllOpen
class QuadTree<T : IQuadTreeObj>(
    val x: Int,
    val y: Int,
    val width: Int,
    val height: Int
) {
    //格子最小宽度(优先级大于最大数目), <=0表示不限制
    private val minWidth = 1

    //格子内最大容纳的物体数目
    private val maxObjCount = 4

    //格子内的物体
    private val objs = LinkedList<T>()

    //子节点 固定4个
    private var cells: List<QuadTree<T>>? = null

    /**
     * 插入对象
     */
    fun insert(obj: T) {
        val posX = obj.fetchPosX()
        val posY = obj.fetchPosY()

        val mCells = cells
        //该格子有子节点 往子节点里面塞
        if (mCells != null) {
            for (i in 0..3) {
                if (mCells[i].containsPos(posX, posY)) {
                    mCells[i].insert(obj)
                }
            }
            return
        }

        //没有子节点 自己存储
        objs.add(obj)

        //大于最大存储数目拆为四个子节点
        if (objs.size > maxObjCount && (minWidth <= 0 || width > minWidth)) {
            //+1是处理临界的情况
            val leftWidth = (width + 1) / 2
            val rightWidth = width - leftWidth
            val topHeight = (height + 1) / 2
            val bottomHeight = height - topHeight
            val newCells = LinkedList<QuadTree<T>>()
            newCells.add(QuadTree(x, y, leftWidth, topHeight))
            newCells.add(QuadTree(x + leftWidth, y, rightWidth, topHeight))
            newCells.add(QuadTree(x + leftWidth, y + topHeight, rightWidth, bottomHeight))
            newCells.add(QuadTree(x, y + topHeight, leftWidth, bottomHeight))
            cells = newCells

            for (i in 1..objs.size) {
                //当前节点数据插入子节点
                insert(objs[objs.size - i])
            }
            //有子节点 本身不在存储
            objs.clear()
        }
    }

    /**
     * 移除对象
     */
    fun remove(obj: T) {
        val posX = obj.fetchPosX()
        val posY = obj.fetchPosY()
        if (!containsPos(posX, posY)) {
            return
        }
        objs.remove(obj)
        val mCells = cells ?: return
        for (i in 0..3) {
            if (mCells[i].containsPos(posX, posY)) {
                mCells[i].remove(obj)
            }
        }
    }

    /**
     * 判断坐标是否在范围内
     */
    private fun containsPos(x: Int, y: Int): Boolean {
        return x >= this.x && x < this.x + width && y >= this.y && y < this.y + height
    }

    /**
     * 根据坐标搜索
     */
    fun search(num: Int, posX: Int, posY: Int, findObj: LinkedList<T>, check: (obj: T) -> Boolean = { _ -> true }) {
        if (findObj.size >= num) {
            return
        }
        val mCells = cells
        if (mCells.isNullOrEmpty()) {
            //添加本体节点
            for (obj in objs) {
                if (check(obj)) {
                    findObj.add(obj)
                    if (findObj.size >= num) {
                        break
                    }
                }
            }
            return
        }

        //优先搜索节点所在区域，再搜索附近区域
        var index = 0
        for (i in 0..3) {
            if (mCells[i].containsPos(posX, posY)) {
                index = i
                break
            }
        }
        for (i in 0..3) {
            mCells[index].search(num, posX, posY, findObj, check)
            index = (index + 1) % 4
        }
    }
}

