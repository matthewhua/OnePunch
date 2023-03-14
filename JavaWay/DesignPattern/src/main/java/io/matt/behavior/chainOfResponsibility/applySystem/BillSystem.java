package io.matt.behavior.chainOfResponsibility.applySystem;

public class BillSystem {

    public static void main(String[] args) {
        TeamLeader 战神 = new TeamLeader("战神");
        Supervisor 周立 = new Supervisor("周立");
        Manager 小渚 = new Manager("小渚");
        Boss 华总 = new Boss("华总");
        战神.setSuperior(周立);
        周立.setSuperior(小渚);
        小渚.setSuperior(华总);


        // 创建报销单
        Bill bill1 = new Bill(1, "Jungle", 8);
        Bill bill2 = new Bill(2, "Lucy", 14.4);
        Bill bill3 = new Bill(3, "Jack", 32.9);
        Bill bill4 = new Bill(4, "Tom", 89);

        // 全部先交给组长审批
        战神.handleRequest(bill1);
        System.out.printf("\n");
        战神.handleRequest(bill2);
        System.out.printf("\n");
        战神.handleRequest(bill3);
        System.out.printf("\n");
        战神.handleRequest(bill4);

    }
}
