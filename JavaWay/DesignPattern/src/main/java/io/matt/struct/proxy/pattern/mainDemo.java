package io.matt.struct.proxy.pattern;

public class mainDemo {

    public static void main(String[] args) {
        Subject exProxy = new ExProxy();
        exProxy.method();
        Subject proxy = new Proxy();
        proxy.method();
    }
}
