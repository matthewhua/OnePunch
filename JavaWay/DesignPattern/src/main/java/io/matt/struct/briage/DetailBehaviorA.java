package io.matt.struct.briage;

public class DetailBehaviorA extends AbstractBehavior {
    @Override
    public void operationOne() {
        System.out.println("op-1 from DetailBehavior!");
    }

    @Override
    public void operationTwo() {
        System.out.println("op-2 from DetailBehaviorA");
    }
}
