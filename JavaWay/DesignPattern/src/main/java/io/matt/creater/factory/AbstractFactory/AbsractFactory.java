package io.matt.creater.factory.AbstractFactory;

/**
 * @author: Matthew
 * @created 2022/12/28 11:25
 */
// 抽象的家具工厂
public abstract class AbsractFactory {
    abstract Chair createChair();
    abstract Sofa createSofa();
    abstract Table createTable();
}
