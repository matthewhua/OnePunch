package io.matt.behavior.visitor;

public class ElementB implements Element{

    private int stateForB = 0;

    @Override
    public void accept(Visitor visitor) {
        System.out.println("=== 开始访问元素B 。。。。。");
        visitor.visitB(this);
    }

    public int getStateForB() {
        return stateForB;
    }

    public void setStateForB(int stateForB) {
        this.stateForB = stateForB;
    }
}
