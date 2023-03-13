package io.matt.behavior.state.game;

public class Professional extends Level {

    public Professional() {
    }


    public Professional(Level level) {
        this.setGameAccount(level.getGameAccount());
    }


    @Override
    void doubleScore() {
        System.out.println("使用胜利双倍积分技能");
    }

    @Override
    void changeCards() {
        System.out.print("使用换牌技能,");
    }

    @Override
    void peekCards() {

    }

    @Override
    void upGraderLevel() {
        if (this.getGameAccount().getScore() < 200) {
            this.getGameAccount().setLevel(new Secondary(this));
            System.out.printf("\t降级！ 级别：Secondary\n\n");
        } else if (this.getGameAccount().getScore() > 250) {
            this.getGameAccount().setLevel(new Final(this));
            System.out.printf("\t升级！ 级别：Final\n\n");
        }
    }
}
