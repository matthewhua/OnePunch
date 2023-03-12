package io.matt.behavior;

import java.util.ArrayList;

public class Demo {

    public static void main(String[] args) {
        ArrayList<Element> elements = new ArrayList<>();
        ElementA elementA = new ElementA();
        elementA.setStateForA(11);
        ElementB elementB = new ElementB();
        elementB.setStateForB(12);
        elements.add(elementA);
        elements.add(elementB);
        for (Element element : elements) {
            element.accept(new VisitorBehavior());
        }
    }
}
