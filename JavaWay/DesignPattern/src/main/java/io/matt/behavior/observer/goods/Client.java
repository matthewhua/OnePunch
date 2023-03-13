package io.matt.behavior.observer.goods;

public class Client {

    public static void main(String[] args) {
        Subscriber subscriber = new Subscriber();
        Subscriber2 subscriber2 = new Subscriber2();
        Subscriber3 subscriber3 = new Subscriber3();
        MessagePublisher messagePublisher = new MessagePublisher();
        messagePublisher.attach(subscriber);
        messagePublisher.attach(subscriber2);
        messagePublisher.notifyUpdate(new Message("我是火华哥")); //subscriber 和 subscriber2会收到消息通知
        messagePublisher.detach(subscriber);
        messagePublisher.attach(subscriber3);
        messagePublisher.notifyUpdate(new Message("我是七元"));  //subscriber2和 subscriber3 会收到消息通知
    }
}
