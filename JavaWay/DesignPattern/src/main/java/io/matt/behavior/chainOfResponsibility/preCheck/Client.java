package io.matt.behavior.chainOfResponsibility.preCheck;

public class Client {

    public static void main(String[] args) {
        Chain chain = new Chain();
        chain.process(99);
        System.out.println("-------------");
        chain.process(-111);
        System.out.println("-------------");
        chain.process(0);
        System.out.println("---------------");
        chain.process(null);
    }
}
