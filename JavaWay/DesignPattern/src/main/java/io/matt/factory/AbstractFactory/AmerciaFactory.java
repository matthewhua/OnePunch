package io.matt.factory.AbstractFactory;

/**
 * @author: Matthew
 * @created 2022/12/28 13:35
 */
// 美国的家具工厂
public class AmerciaFactory extends AbsractFactory {

    @Override
    Chair createChair() {
        return new AmerciaChair();
    }

    @Override
    Sofa createSofa() {
        return new AmercianSofa();
    }

    @Override
    Table createTable() {
        return new AmericaTable();
    }
}
