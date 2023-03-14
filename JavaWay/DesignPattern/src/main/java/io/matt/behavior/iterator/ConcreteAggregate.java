package io.matt.behavior.iterator;

public class ConcreteAggregate implements Aggregate {

    private Object[] objects;

    public ConcreteAggregate(Object[] objects) {
        this.objects = objects;
    }

    @Override
    public Iterator createIterator() {
        return new ConcreteIterator(objects);
    }
}
