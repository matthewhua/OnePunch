package io.matt.behavior.visitor.pattern;


import io.matt.behavior.visitor.Visitor;

public class Apple extends BaseElement {

    public Apple() {
        this.setName("");
    }

    public Apple(String name, int price) {
        // 链式调用
        this.setName(name).setPrice(price);
    }

    @Override
    public void accept(Visitor visitor) {
        visitor.visit(this);
    }
}
