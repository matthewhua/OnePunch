package io.matt.behavior.chainOfResponsibility.applySystem;


// 具体处理者：组长
public class TeamLeader extends Approver {

    public TeamLeader() {
    }

    public TeamLeader(String name) {
        super(name);
    }

    @Override
    void handleRequest(Bill bill) {
        if (bill.getAccount() < 10) {
            System.out.printf("组长 %s 处理了该票据，票据信息：", this.getName());
            System.out.println(bill);
        } else {
            System.out.printf("组长无权处理，转交上级……\n");
            this.getSuperior().handleRequest(bill);
        }
    }
}
