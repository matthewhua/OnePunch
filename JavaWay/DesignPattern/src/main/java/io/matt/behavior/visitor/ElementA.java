package io.matt.behavior.visitor;

public class ElementA implements Element {

    private int stateForA = 0;


    @Override
    public void accept(Visitor visitor) {
        System.out.println("=== 开始访问元素A 。。。。。");
        visitor.visitA(this);
    }


    public int getStateForA() {
        return stateForA;
    }

    public void setStateForA(int stateForA) {
        this.stateForA = stateForA;
    }
}
