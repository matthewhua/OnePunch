package io.matt.factory.FactoryMethod;

/**
 * @author: Matthew
 * @created 2022/12/28 14:49
 */
public class Client {

    public static void main(String[] args) {
        final IProduct product = ProductFactory.getProduct("");
        product.apply();
        final IProduct a = ProductFactory.getProduct("a");
        a.apply();
    }
}
