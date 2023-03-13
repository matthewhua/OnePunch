package io.matt.behavior.state.game;

public class Secondary extends Level {

    public Secondary() {
    }

    public Secondary(Level level) {
        this.setGameAccount(level.getGameAccount());
    }

    @Override
    void doubleScore() {
        System.out.println("使用胜利双倍积分技能");
    }

    @Override
    void changeCards() {

    }

    @Override
    void peekCards() {

    }

    @Override
    void upGraderLevel() {
        if (this.getGameAccount().getScore() < 150) {
            this.getGameAccount().setLevel(new Primary(this));
            System.out.printf("\t降级！ 级别：PRIMARY\n\n");
        } else if (this.getGameAccount().getScore() > 200) {
            this.getGameAccount().setLevel(new Professional(this));
            System.out.printf("\t升级！ 级别：PROFESSIONAL\n\n");
        }
    }
}
