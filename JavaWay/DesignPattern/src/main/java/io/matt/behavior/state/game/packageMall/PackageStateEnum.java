package io.matt.behavior.state.game.packageMall;

public enum PackageStateEnum {
    ACKNOW(1, "已下单"),
    WAREHOUSE(2, "仓库处理中"),
    IN_TRANSITION(3, "运输中"),
    DELIVERING(4, "派送中"),
    WAIT_FOR_PICKUP(5, "待取件"),
    RECEIVED(6, "已签收");

    private int id;
    private String desc;

    PackageStateEnum(int id, String desc) {
        this.id = id;
        this.desc = desc;
    }

    public int getId() {
        return id;
    }
}
