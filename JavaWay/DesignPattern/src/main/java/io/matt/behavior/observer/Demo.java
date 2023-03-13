package io.matt.behavior.observer;

public class Demo {


    public static void main(String[] args) {
        PublisherImpl account = new PublisherImpl("Test123", 10.00);
        ObserverImpl observer = new ObserverImpl();
        account.addObserver(observer);
        account.notify(11.00);
    }
}
