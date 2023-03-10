package io.matt.build.Product;

/**
 * @author: Matthew
 * @created 2022/12/26 17:32
 */
public interface Builder {
    Builder buildPartA(int partA);
    Builder buildPartB(String partB);
    Builder buildPartC(int partC);
    Product build();
}
