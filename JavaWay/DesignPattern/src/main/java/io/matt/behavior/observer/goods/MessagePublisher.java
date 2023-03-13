package io.matt.behavior.observer.goods;

import java.util.ArrayList;
import java.util.List;

public class MessagePublisher implements Subject {

    private List<MessageObserver> observers = new ArrayList<>();

    @Override
    public void attach(MessageObserver observer) {
        observers.add(observer);
    }

    @Override
    public void detach(MessageObserver observer) {
        observers.remove(observer);
    }

    @Override
    public void notifyUpdate(Message message) {
        observers.forEach(x -> x.update(message));
    }
}
