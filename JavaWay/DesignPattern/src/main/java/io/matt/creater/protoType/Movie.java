package io.matt.creater.protoType;

public class Movie implements IPrototype{

    @Override
    public Movie clone() throws CloneNotSupportedException {
        System.out.println("Cloning Movie object..");
        return (Movie) super.clone();
    }

    //方便结果展示
    @Override
    public String toString() {
        return "Movie{}";
    }
}
