package io.matt.behavior.visitor.pattern;


import io.matt.behavior.visitor.ElementA;
import io.matt.behavior.visitor.ElementB;
import io.matt.behavior.visitor.Visitor;

public abstract class BaseVisitor implements Visitor{
    @Override
    public void visitA(ElementA elementA) {

    }

    @Override
    public void visitB(ElementB elementB) {

    }

}

class Customer extends BaseVisitor {

    private String name;

    public Customer() {
        this.name = "";
    }

    public Customer(String name) {
        this.name = name;
    }

    public void setNum(Apple apple, int num) {
        apple.setNum(num);
    }

    public void setNum(Book book, int num) {
        book.setNum(num);
    }


    @Override
    public void visit(Apple apple) {
        int price = apple.getPrice();
        System.out.printf(" %s \t 单价: \t%d 元/kg\n", apple.getName(), apple.getPrice());
    }

    @Override
    public void visit(Book book) {
        int price = book.getPrice();
        String name = book.getName();
        System.out.printf("《%s》\t单价: \t%d 元/本\n", name, price);
    }
}

class Cashier extends BaseVisitor {


    @Override
    public void visit(Apple apple) {
        String name = apple.getName();
        int price = apple.getPrice();
        int num = apple.getNum();
        int total = price*num;
        System.out.printf("  %s 总价： %d 元\n", name, total);
    }

    @Override
    public void visit(Book book) {
        int price = book.getPrice();
        String name = book.getName();
        int num = book.getNum();
        int total = price*num;
        System.out.printf("  《%s》 总价： %d 元\n", name, total);
    }
}