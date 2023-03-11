package io.matt.struct.decorator;

// 装饰器
public class BaseDecorator implements Component {
    private Component wrapper;

    public BaseDecorator(Component wrapper) {
        this.wrapper = wrapper;
    }

    @Override
    public void execute() {
        wrapper.execute();
    }
}

//具体装饰器A
class DecoratorA extends BaseDecorator {
    public DecoratorA(Component wrapper) {
        super(wrapper);
    }

}

//具体装饰器B
class DecoratorB extends BaseDecorator {
    public DecoratorB(Component wrapper) {
        super(wrapper);
    }


}