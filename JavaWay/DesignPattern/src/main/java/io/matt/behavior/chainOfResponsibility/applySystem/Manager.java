package io.matt.behavior.chainOfResponsibility.applySystem;

public class Manager extends Approver {

    public Manager(String name) {
        super(name);
    }

    @Override
    void handleRequest(Bill bill) {
        if (bill.getAccount() >= 30 && bill.getAccount() < 60) {
            System.out.printf("经理 %s 处理了该票据，票据信息：", this.getName());
            System.out.println(bill);
        } else {
            System.out.print("经理无权处理，转交上级……\n");
            this.getSuperior().handleRequest(bill);
        }
    }
}
