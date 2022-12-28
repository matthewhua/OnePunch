package io.matt.factory.AbstractFactory;

/**
 * @author: Matthew
 * @created 2022/12/28 11:17
 */
public class Client {
    private Chair chair;
    private Sofa mySofa;
    private Table myTable;

    public Client(Chair chair, Sofa mySofa, Table myTable) {
        this.chair = chair;
        this.mySofa = mySofa;
        this.myTable = myTable;
    }
}
