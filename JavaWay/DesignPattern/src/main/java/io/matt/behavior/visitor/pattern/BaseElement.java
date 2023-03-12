package io.matt.behavior.visitor.pattern;

import io.matt.behavior.visitor.Element;

public abstract class BaseElement implements Element {

    private int price;
    private int num;
    private String name;

    public int getPrice() {
        return price;
    }

    public BaseElement setPrice(int price) {
        this.price = price;
        return this;
    }

    public int getNum() {
        return num;
    }

    public BaseElement setNum(int num) {
        this.num = num;
        return this;
    }

    public String getName() {
        return name;
    }

    public BaseElement setName(String name) {
        this.name = name;
        return this;
    }
}


