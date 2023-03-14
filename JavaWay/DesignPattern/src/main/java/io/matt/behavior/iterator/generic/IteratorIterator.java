package io.matt.behavior.iterator.generic;

public interface IteratorIterator<E> {

    void reset();                   // 重置为第一个元素

    E next();                      //获取下一个元素

    E currentItem();               //检索当前元素

    boolean hasNext();             //判断是否还有下一个元素存在.
}
