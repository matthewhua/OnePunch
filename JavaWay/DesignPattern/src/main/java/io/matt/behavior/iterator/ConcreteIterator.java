package io.matt.behavior.iterator;

public class ConcreteIterator implements Iterator {

    private Object[] objects;
    private int position;

    public ConcreteIterator(Object[] objects) {
        this.objects = objects;
    }

    @Override
    public Object next() {
        return objects[position++];
    }

    @Override
    public boolean hasNext() {
        return position < objects.length;
    }
}
