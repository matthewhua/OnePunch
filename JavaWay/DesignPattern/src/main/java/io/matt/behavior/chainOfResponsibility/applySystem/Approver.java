package io.matt.behavior.chainOfResponsibility.applySystem;


// 抽象处理者
public abstract class Approver {

    private Approver superior;
    private String name;

    public Approver() {
    }

    public Approver(String name) {
        this.name = name;
    }

    abstract void handleRequest(Bill bill);

    public Approver getSuperior() {
        return superior;
    }

    public void setSuperior(Approver superior) {
        this.superior = superior;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }
}
