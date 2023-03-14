package io.matt.behavior.iterator;

public class Demo {

    public static void main(String[] args) {
        Object[] objects = new Object[2];
        objects[0] = new Object();
        objects[1] = new Object();
        ConcreteAggregate aggregate = new ConcreteAggregate(objects);
        Iterator iterator = aggregate.createIterator();
        while (iterator.hasNext()) {
            Object next = iterator.next();
            System.out.println(next.toString());
        }
    }
}
