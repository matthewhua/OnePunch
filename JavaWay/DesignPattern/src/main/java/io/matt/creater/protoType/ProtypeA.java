package io.matt.creater.protoType;

public class ProtypeA implements ProtoTypeInterface{

    @Override
    public ProtypeA clone() throws CloneNotSupportedException {
        System.out.println("Cloning new object: A");
        return (ProtypeA) super.clone();
    }
}
