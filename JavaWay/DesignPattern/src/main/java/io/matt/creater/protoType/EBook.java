package io.matt.creater.protoType;

public class EBook implements IPrototype{

    @Override
    public EBook clone() throws CloneNotSupportedException {
        System.out.println("Cloning Book object..");
        return (EBook) super.clone();
    }

    @Override
    public String toString() {
        return "EBook{}";
    }
}
