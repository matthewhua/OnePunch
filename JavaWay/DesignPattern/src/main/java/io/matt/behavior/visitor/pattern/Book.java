package io.matt.behavior.visitor.pattern;

import io.matt.behavior.visitor.Visitor;

public class Book extends BaseElement {

    public Book() {
        this.setName("");
    }

    public Book(String name, int price) {
        // 链式调用
        this.setName(name).setPrice(price);
    }

    @Override
    public void accept(Visitor visitor) {
        visitor.visit(this);
    }
}
