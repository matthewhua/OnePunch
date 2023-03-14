package io.matt.behavior.mediator;

public class Demo {


    public static void main(String[] args) {
        MediatorImpl mediator = new MediatorImpl();
        ComponentA componentA = new ComponentA(mediator);
        componentA.exec("key - A");
        ComponentB componentB = new ComponentB(mediator);
        componentB.exec("Key - B");
    }
}
