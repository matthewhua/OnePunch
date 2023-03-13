package io.matt.behavior.observer.pubg;


// 抽象观察者 Observer
public interface GameObserver {

    void call(INFO_TYPE infoType, AllyCenter ac);

    String getName();
}

abstract class BaseObserver implements GameObserver {
    private String name;

    @Override
    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }
}

// 具体观察者
class Player extends BaseObserver {

    public Player() {
        setName("火华哥");
    }

    public Player(String name) {
        setName(name);
    }


    @Override
    public void call(INFO_TYPE infoType, AllyCenter ac) {
        switch (infoType) {
            case HELP:
                System.out.printf("%s : 救救我\n", getName());
                break;
            case RESOURCE:
                System.out.printf("%s :我这里有物资\n", getName());
                break;
            default:
                System.out.print("Nothing\n");
        }
        ac.notify(infoType, getName());
    }

    // 实现具体方法
    public void help() {
        System.out.printf("%s:坚持住，我来救你！\n", getName());
    }

    public void come() {
        System.out.printf("%s:好的，我来取物资\n", getName());
    }
}
