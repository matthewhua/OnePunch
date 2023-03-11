package io.matt.creater.protoType;

public class IPrototypeDemo {

    public static void main(String[] args) throws CloneNotSupportedException {
        String movieProtoType = PrototypeFactory.getInstance(ModelType.MOVIE.getName()).toString();
        System.out.println(movieProtoType);
        String eBookPrototype  = PrototypeFactory.getInstance(ModelType.EBOOK.getName()).toString();
        System.out.println(eBookPrototype);
    }
}
