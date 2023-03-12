package io.matt.struct.flyweight;

public class MainDemo {

    public static void main(String[] args) {
        FlyweightFactory factory = FlyweightFactory.getInstance();
        Flyweight a = factory.getFlyweight("A");
        a.operation(1);
        Flyweight b = factory.getFlyweight("B");
        b.operation(3);
        Flyweight 施鸿烨 = factory.getFlyweight("施鸿烨");
        施鸿烨.operation(1);
    }
}
