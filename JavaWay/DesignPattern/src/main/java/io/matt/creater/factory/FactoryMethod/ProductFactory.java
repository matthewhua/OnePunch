package io.matt.creater.factory.FactoryMethod;

/**
 * @author: Matthew
 * @created 2022/12/28 14:47
 */
// 核心工厂类
public class ProductFactory {

    public static IProduct getProduct(String name) {
        if ("a".equals(name)) {
            return new Product_A_Impl();
        }
        return new Product_B_Impl();
    }
}

class Product_A_Impl implements IProduct {

    @Override
    public void apply() {
        System.out.println("use A product now");
    }
}

class Product_B_Impl implements IProduct {

    @Override
    public void apply() {
        System.out.println("use B product now");
    }
}


