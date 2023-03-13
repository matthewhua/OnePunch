package io.matt.behavior.state.game;

public class Primary extends Level {

    public Primary() {
    }

    public Primary(GameAccount gameAccount) {
        this.setGameAccount(gameAccount);
    }

    public Primary(Level level) {
        this.setGameAccount(level.getGameAccount());
    }


    @Override
    void doubleScore() {

    }

    @Override
    void changeCards() {

    }

    @Override
    void peekCards() {

    }

    @Override
    void upGraderLevel() {
        if (this.getGameAccount().getScore() > 150) {
            this.getGameAccount().setLevel(new Secondary(this));
            System.out.printf("\t升级！！ 级别：SECONDARY\n\n");
        } else {
            System.out.printf("\n");
        }
    }


}
