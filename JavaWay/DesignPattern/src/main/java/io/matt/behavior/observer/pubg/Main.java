package io.matt.behavior.observer.pubg;

public class Main {

    public static void main(String[] args) {
        // 创建一个战队
        AllyCenterController allyCenterController = new AllyCenterController();

        // 创建4个玩家，并加入战队
        Player jungle = new Player("Jungle");
        Player single = new Player("Single");
        Player 火华哥 = new Player("火华哥");
        Player matthew = new Player("Matthew");

        allyCenterController.join(jungle);
        allyCenterController.join(single);
        allyCenterController.join(火华哥);
        allyCenterController.join(matthew);

        System.out.printf("\n\n");

        // Jungle 发现物资， 呼叫队友
        matthew.call(INFO_TYPE.RESOURCE, allyCenterController);

        System.out.printf("\n\n");

        // 傻子狗遇到危险, 求救队友
        matthew.call(INFO_TYPE.HELP, allyCenterController);

        System.out.printf("\n\n");
    }
}
