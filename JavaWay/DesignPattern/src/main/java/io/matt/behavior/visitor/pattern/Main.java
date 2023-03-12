package io.matt.behavior.visitor.pattern;

public class Main {

    public static void main(String[] args) {
        Apple apple1 = new Apple("红富士苹果", 6);
        Apple apple2 = new Apple("花牛苹果", 4);
        Book book1 = new Book("红楼梦", 129);
        Book book2 = new Book("西游记", 23);

        Cashier cashier = new Cashier();
        Customer jungle = new Customer("jungle");
        jungle.setNum(apple1, 2);
        jungle.setNum(apple2, 4);
        jungle.setNum(book1, 1);
        jungle.setNum(book2, 3);

        ShoppingCart shoppingCart = new ShoppingCart();
        shoppingCart.addElement(apple1);
        shoppingCart.addElement(apple2);
        shoppingCart.addElement(book1);
        shoppingCart.addElement(book2);

        System.out.printf("\n\n");
        shoppingCart.accept(jungle);


        System.out.printf("\n\n");
        shoppingCart.accept(cashier);
    }
}
