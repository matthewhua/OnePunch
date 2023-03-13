package io.matt.behavior.observer.goods;

/**
 * 被观察者
 */
public interface Subject {

    void attach(MessageObserver observer);

    void detach(MessageObserver observer);

    void notifyUpdate(Message message); //更新通知
}
