package io.matt.behavior.state.game;

import java.util.concurrent.ThreadLocalRandom;

public class GameAccount {
    private Level level;
    private int score;
    private String name;

    public GameAccount() {
        System.out.print("创立游戏角色，积分：100，级别：PRIMARY\n");
        this.score = 100;
        this.name = "visitor";
        setLevel(new Primary(this));
    }

    public GameAccount(String iName) {
        System.out.print("创立游戏角色，积分：100，级别：PRIMARY\n");
        this.name = iName;
        this.score = 100;
        setLevel(new Primary(this));
    }

    public void playCard() {
        this.level.playCard();
        try {
            Thread.sleep(100);
        } catch (InterruptedException e) {
            throw new RuntimeException(e);
        }
        int res = ThreadLocalRandom.current().nextInt(0, 2);
        if (res % 2 == 0) {
            this.win();
        } else {
            this.lose();
        }
        this.level.upGraderLevel();
    }

    void win() {
        if (this.getScore() < 200) {
            setScore(getScore() + 50);
        } else {
            setScore(getScore() + 100);
        }
        System.out.printf("\n\t胜利，最新积分为 %d\n", score);
    }

    void lose() {
        setScore(getScore() + 30);
        System.out.printf("\n\t输牌，最新积分为 %d\n", score);
    }

    public Level getLevel() {
        return level;
    }

    public void setLevel(Level level) {
        this.level = level;
    }

    public int getScore() {
        return score;
    }

    public void setScore(int score) {
        this.score = score;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }
}
