package io.matt.behavior.strategy.promotion;

public class Promotional {

    private final PromotionStrategy strategy;

    public Promotional(PromotionStrategy strategy) {
        this.strategy = strategy;
    }

    public void recommend(String skuId) {
        strategy.recommend(skuId);
    }
}
