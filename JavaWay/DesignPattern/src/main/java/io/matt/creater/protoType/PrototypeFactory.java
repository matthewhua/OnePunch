package io.matt.creater.protoType;

import java.util.HashMap;
import java.util.Map;

public class PrototypeFactory {

    //这里充当注册表的作用，用于存放原始对象，作为对象拷贝的基础
    private static Map<String, IPrototype> prototypes = new HashMap<>();
    //初始化时就将原始对象放入注册表中
    static {
        prototypes.put(ModelType.MOVIE.getName(), new Movie());
        prototypes.put(ModelType.EBOOK.getName(), new EBook());
    }

    //获取对象时，是使用name来进行对象拷贝
    public static IPrototype getInstance(final String s) throws CloneNotSupportedException {
        return prototypes.get(s).clone();
    }
}
