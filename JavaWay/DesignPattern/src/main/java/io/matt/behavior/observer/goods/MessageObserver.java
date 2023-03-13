package io.matt.behavior.observer.goods;


/**
 * 观察者 当有消息 Message 发送过来时就会调用该方法
 */
public interface MessageObserver {

    void update(Message message);
}
