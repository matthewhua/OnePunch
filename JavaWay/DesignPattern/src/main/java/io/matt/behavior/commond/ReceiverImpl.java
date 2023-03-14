package io.matt.behavior.commond;

public class ReceiverImpl implements Receiver {

    @Override
    public void operationA() {
        System.out.println("操作 A");
    }

    @Override
    public void operationB() {
        System.out.println("操作 B");
    }

    @Override
    public void operationC() {
        System.out.println("操作 C");
    }
}
