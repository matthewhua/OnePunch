package io.matt.behavior.state.game;

public class Final extends Level {

    public Final() {
    }

    public Final(Level level) {
        this.setGameAccount(level.getGameAccount());
    }

    @Override
    void doubleScore() {
        System.out.print("使用胜利双倍积分技能,");
    }

    @Override
    void changeCards() {
        System.out.print("使用换牌技能,");
    }

    @Override
    void peekCards() {
        System.out.print("使用偷看卡牌技能");
    }

    @Override
    void upGraderLevel() {
        if (this.getGameAccount().getScore() < 250) {
            this.getGameAccount().setLevel(new Professional(this));
            System.out.print("\t降级！ 级别：PROFESSIONAL\n\n");
        } else {
            System.out.printf("\t%s 已经是最高级\n\n", this.getGameAccount().getName());
        }
    }
}
