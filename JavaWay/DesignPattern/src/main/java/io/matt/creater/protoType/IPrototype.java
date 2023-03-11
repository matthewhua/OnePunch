package io.matt.creater.protoType;

public interface IPrototype extends Cloneable{

    //继承Cloneable接口，重写clone()方法便能使用父类的Object.clone()复制方法
    //也可以直接实现Cloneable接口，效果一样。这里我们为了统一业务接口层级，子类都实现IPrototype接口
    IPrototype clone() throws CloneNotSupportedException;
}


enum ModelType {
    MOVIE("movie"),
    EBOOK("eBook");
    private String name;
    ModelType(String name) {
        this.name = name;
    }
    public String getName() {
        return name;
    }
}

