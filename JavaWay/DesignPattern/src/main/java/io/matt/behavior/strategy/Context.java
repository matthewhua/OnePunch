package io.matt.behavior.strategy;

public class Context {

    public void request(Strategy strategy) {
        strategy.operation();
    }
}
