package io.matt.behavior.visitor;

import io.matt.behavior.visitor.pattern.Apple;
import io.matt.behavior.visitor.pattern.Book;

public class VisitorBehavior implements Visitor {

    @Override
    public void visitA(ElementA elementA) {
        int x = elementA.getStateForA();
        x++;
        System.out.println("=== 当前A的state为：" + x);
        elementA.setStateForA(x);
    }

    @Override
    public void visitB(ElementB elementB) {
        int x = elementB.getStateForB();
        x++;
        System.out.println("=== 当前B的state为：" + x);
        elementB.setStateForB(x);
    }

    @Override
    public void visit(Apple apple) {

    }

    @Override
    public void visit(Book book) {

    }
}
