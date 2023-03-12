package io.matt.behavior.strategy;

public class Main {

    public static void main(String[] args) {
        Context context = new Context();
        context.request(new StrategyA());
        context.request(new StrategyB());
    }
}
