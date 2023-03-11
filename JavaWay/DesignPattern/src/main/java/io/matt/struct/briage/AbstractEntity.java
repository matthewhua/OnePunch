package io.matt.struct.briage;

public abstract class AbstractEntity {

    // 行为对象
    protected AbstractBehavior myBehavior;

    //实体与行为的关联
    public AbstractEntity(AbstractBehavior myBehavior) {
        this.myBehavior = myBehavior;
    }

    // 子类需要实现的方法
    public abstract void request();
}
