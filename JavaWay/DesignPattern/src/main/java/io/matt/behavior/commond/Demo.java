package io.matt.behavior.commond;

public class Demo {


    public static void main(String[] args) {
        ReceiverImpl receiver = new ReceiverImpl();
        Invoker invoker = new Invoker();
        invoker.setCommand(new Command1(receiver), new Command2(receiver), new Command3(receiver));
        invoker.run();

    }
}
