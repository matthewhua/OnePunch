package io.matt.behavior.chainOfResponsibility.applySystem;

public class Boss extends Approver {


    public Boss(String name) {
        super(name);
    }

    @Override
    void handleRequest(Bill bill) {
        System.out.printf("老板 %s 处理了该票据, 票据信息：", this.getName());
        System.out.println(bill);
    }
}
