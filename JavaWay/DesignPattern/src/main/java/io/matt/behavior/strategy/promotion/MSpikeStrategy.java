package io.matt.behavior.strategy.promotion;

public class MSpikeStrategy implements PromotionStrategy {

    @Override
    public int recommend(String skuId) {
        System.out.println("==== 执行 M 元 秒杀活动");
        // 推荐算法和罗杰写这里
        return 3;
    }
}
