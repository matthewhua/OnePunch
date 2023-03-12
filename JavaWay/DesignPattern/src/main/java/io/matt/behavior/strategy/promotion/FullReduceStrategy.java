package io.matt.behavior.strategy.promotion;

public class FullReduceStrategy implements PromotionStrategy {

    @Override
    public int recommend(String skuId) {
        System.out.println("=== 执行 满减活动");
        //推荐算法和逻辑写这里
        return 1;
    }
}
