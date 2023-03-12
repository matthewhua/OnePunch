package io.matt.behavior.visitor;

public interface Element {

    void accept(Visitor visitor);
}
