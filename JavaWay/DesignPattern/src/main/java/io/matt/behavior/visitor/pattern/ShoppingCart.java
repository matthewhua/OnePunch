package io.matt.behavior.visitor.pattern;

import io.matt.behavior.visitor.Visitor;

import java.util.ArrayList;
import java.util.List;

public class ShoppingCart {

    List<BaseElement> elements = new ArrayList<>();

    void addElement(BaseElement baseElement) {
        System.out.printf(" 商店名： %s, \t 数量 ：%d, \t 加入购物车成功! \n", baseElement.getName(), baseElement.getNum());
        elements.add(baseElement);
    }

    void accept(Visitor visitor) {
        for (BaseElement element : elements) {
            element.accept(visitor);
        }
    }


}
