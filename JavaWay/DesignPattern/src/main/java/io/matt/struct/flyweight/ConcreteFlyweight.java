package io.matt.struct.flyweight;

public class ConcreteFlyweight implements Flyweight {
    private String uniqueKey;

    public ConcreteFlyweight(String uniqueKey) {
        this.uniqueKey = uniqueKey;
    }

    @Override
    public void operation(int state) {
        System.out.printf("=== 享元内部状态： &s, 外部状态: %s%n", uniqueKey, state);
    }
}
