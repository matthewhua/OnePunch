package xyz.ariane.util.math

import com.google.common.base.Preconditions
import com.google.common.math.DoubleMath

import java.math.RoundingMode

object PriceUtils {

    /**
     * 道具卖出价格=max（1，四舍五入（单价*道具数量*折扣））
     */
    fun calcItemPrice(unitPrice: Double, num: Int, discount: Int): Int {
        Preconditions.checkArgument(num > 0, "道具数量需>0")
        Preconditions.checkArgument(discount > 0, "折扣需>0")
        val price = unitPrice * discount / 100 * num
        val priceInt = DoubleMath.roundToInt(price, RoundingMode.HALF_UP)
        return if (priceInt > 0) priceInt else Integer.MAX_VALUE
    }

}
