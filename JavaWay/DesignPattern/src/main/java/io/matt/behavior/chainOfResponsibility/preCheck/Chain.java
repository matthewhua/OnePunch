package io.matt.behavior.chainOfResponsibility.preCheck;

public class Chain {

    Executor chain;

    public Chain() {
        buildChain();
    }

    private void buildChain() {
        NegativeExecutor negativeExecutor = new NegativeExecutor();
        ZeroExecutor zeroExecutor = new ZeroExecutor();
        PositiveExecutor positiveExecutor = new PositiveExecutor();
        negativeExecutor.setNext(zeroExecutor);
        zeroExecutor.setNext(positiveExecutor);
        this.chain = negativeExecutor;
    }

    public void process(Integer num) {
        chain.handle(num);
    }
}
