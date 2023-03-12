package io.matt.behavior.strategy.promotion;

public class NPriceDiscountStrategy implements PromotionStrategy{

    @Override
    public int recommend(String skuId) {
        System.out.println("=== 执行 N 折扣优惠活动");
        //推荐算法和逻辑写这里
        return 2;
    }
}
