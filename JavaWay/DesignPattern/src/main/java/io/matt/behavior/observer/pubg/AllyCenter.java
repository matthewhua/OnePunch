package io.matt.behavior.observer.pubg;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

public interface AllyCenter {

    void notify(INFO_TYPE info_type, String name);

    void join(GameObserver observer);

    void remove(GameObserver observer);
}

abstract class BaseAllyCenter implements AllyCenter {

    protected List<GameObserver> players = new ArrayList<>();

    public BaseAllyCenter() {
        System.out.println("大吉大利，今晚吃鸡!");
    }

    @Override
    public void join(GameObserver observer) {
        if (players.size() >= 4) {
            System.out.println("玩家已满");
            return;
        }
        System.out.printf("玩家 %s 加入\n", observer.getName());

    }

    @Override
    public void remove(GameObserver observer) {
        System.out.printf("玩家 %s 退出\n", observer.getName());
        players.remove(observer);
    }
}

class AllyCenterController extends BaseAllyCenter {

    @Override
    public void notify(INFO_TYPE info_type, String name) {
        switch (info_type) {
            case HELP:
                for (GameObserver player : players) {
                    if (!Objects.equals(player.getName(), name)) {
                        ((Player) player).come();
                    }
                }
                break;
            case RESOURCE:
                for (GameObserver player : players) {
                    if (!Objects.equals(player.getName(), name)) {
                        ((Player) player).help();
                    }
                }
                break;
            default:
                System.out.printf("Noting\n");
        }
    }

}
