package io.matt.behavior.strategy.promotion;

public interface PromotionStrategy {

    /**
     * 返回1 代表 可以参加 满减活动
     * 返回2 代表 可以参加 N折优惠活动
     * 返回3 代表 可以参加 M元秒杀活动
     */
    int recommend(String skuId);
}
