package io.matt.behavior.chainOfResponsibility.applySystem;


// 具体处理者：主管
public class Supervisor extends Approver {

    public Supervisor(String name) {
        super(name);
    }

    @Override
    void handleRequest(Bill bill) {
        if (bill.getAccount() >= 10 && bill.getAccount() < 30) {
            System.out.printf("主管 %s 处理了该票据，票据信息：", this.getName());
            System.out.println(bill);
        } else {
            System.out.print("主管无权处理，转交上级……\n");
            this.getSuperior().handleRequest(bill);
        }
    }

}
