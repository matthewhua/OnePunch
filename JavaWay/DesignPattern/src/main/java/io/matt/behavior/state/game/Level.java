package io.matt.behavior.state.game;

public abstract class Level {

    // 声明方法
    public void playCard() {
        this.play();
        this.doubleScore();
        this.changeCards();
        this.peekCards();
    }

    void play() {
        System.out.println("\t使用基本技能");
    }

    abstract void doubleScore();

    abstract void changeCards();

    abstract void peekCards();

    // 升级
    abstract void upGraderLevel();

    public GameAccount getGameAccount() {
        return gameAccount;
    }

    public void setGameAccount(GameAccount gameAccount) {
        this.gameAccount = gameAccount;
    }

    private GameAccount gameAccount;
}

