package io.matt.creater.factory.AbstractFactory;

/**
 * @author: Matthew
 * @created 2022/12/28 13:23
 */
// 中国的家具工厂
public class ChinaFactory extends AbsractFactory{

    @Override
    Chair createChair() {
        return new ChinaChair();
    }

    @Override
    Sofa createSofa() {
        return new ChinaSofa();
    }

    @Override
    Table createTable() {
        return new ChinaTable();
    }
}
