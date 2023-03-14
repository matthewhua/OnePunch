package io.matt.behavior.mediator;

public interface Mediator {

    void apply(String key);
}

class MediatorImpl implements Mediator {

    @Override
    public void apply(String key) {
        System.out.println("最终中介者执行操作, key 为：" + key);
    }
}

abstract class Component {
    private Mediator mediator;

    public Component(Mediator mediator) {
        this.mediator = mediator;
    }

    public abstract void exec(String key);

    public Mediator getMediator() {
        return mediator;
    }
}

class ComponentA extends Component {
    public ComponentA(Mediator mediator) {
        super(mediator);
    }

    @Override
    public void exec(String key) {
        System.out.println("===在组件A中, 通过中介者执行");
        getMediator().apply(key);
    }
}

class ComponentB extends Component {

    public ComponentB(Mediator mediator) {
        super(mediator);
    }

    @Override
    public void exec(String key) {
        System.out.println("===在组件B中, 通过中介者执行");
        getMediator().apply(key);
    }
}