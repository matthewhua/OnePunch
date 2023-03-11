package io.matt.creater.protoType;

public class ProtypeB implements ProtoTypeInterface{

    @Override
    public ProtypeB clone() throws CloneNotSupportedException {
        System.out.println("Cloning new object: B");
        return (ProtypeB) super.clone();
    }
}
