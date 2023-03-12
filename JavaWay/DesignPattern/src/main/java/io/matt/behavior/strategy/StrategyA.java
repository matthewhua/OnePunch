package io.matt.behavior.strategy;

public class StrategyA implements Strategy {
    @Override
    public void operation() {
        System.out.println("==== 执行策略 A ========");
    }
}
