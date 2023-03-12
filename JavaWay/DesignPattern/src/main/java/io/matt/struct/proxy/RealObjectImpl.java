package io.matt.struct.proxy;

public class RealObjectImpl implements RealObject{

    @Override
    public void doSomething() {
        System.out.println("======== 真实对象输出打印");
    }
}
