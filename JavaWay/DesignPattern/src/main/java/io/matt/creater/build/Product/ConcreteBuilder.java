package io.matt.creater.build.Product;

/**
 * @author: Matthew
 * @created 2022/12/26 17:35
 */
public class ConcreteBuilder implements Builder{

    private int partA;
    private String partB;
    private int partC;

    @Override
    public Builder buildPartA(int partA) {
        this.partA = partA;
        return this;
    }

    @Override
    public Builder buildPartB(String partB) {
        this.partB = partB;
        return this;
    }

    @Override
    public Builder buildPartC(int partC) {
        this.partC = partC;
        return this;
    }

    @Override
    public Product build() {
        return new Product(partA, partB, partC);
    }
}
