package io.matt.behavior.observer.goods;

public class Subscriber implements MessageObserver {
    @Override
    public void update(Message message) {
        System.out.println("MessageSubscriber1 ::" + message.getContent());
    }
}


class Subscriber2 implements MessageObserver {
    @Override
    public void update(Message message) {
        System.out.println("MessageSubscriber2 :: " + message.getContent());
    }
}


class Subscriber3 implements MessageObserver {
    @Override
    public void update(Message message) {
        System.out.println("MessageSubscriber3 :: " + message.getContent());
    }
}