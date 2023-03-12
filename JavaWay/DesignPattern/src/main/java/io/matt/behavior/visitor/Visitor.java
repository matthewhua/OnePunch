package io.matt.behavior.visitor;

import io.matt.behavior.visitor.pattern.Apple;
import io.matt.behavior.visitor.pattern.Book;


public interface Visitor {

    void visitA(ElementA elementA);

    void visitB(ElementB elementB);

    void visit(Apple apple);

    void visit(Book book);
}
