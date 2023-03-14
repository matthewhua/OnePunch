package io.matt.behavior.chainOfResponsibility.applySystem;

// 请求：票据
public class Bill {

    private int io;
    private String name;
    private double account;

    public Bill() {
    }

    public Bill(int io, String name, double account) {
        this.io = io;
        this.name = name;
        this.account = account;
    }

    public double getAccount() {
        return account;
    }

    @Override
    public String toString() {
        return "Bill{" +
                "io=" + io +
                ", name='" + name + '\'' +
                ", account=" + account +
                '}';
    }
}
