package io.matt.creater.protoType;

public class MainDemo {

    public static void main(String[] args) throws CloneNotSupportedException {
        ProtypeA source = new ProtypeA();
        System.out.println(source);

        ProtypeA newInstanceA = source.clone();
        System.out.println(newInstanceA);
    }
}
