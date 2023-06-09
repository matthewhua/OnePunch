package io.matt.oop.superDemo;

import java.lang.reflect.InvocationTargetException;
import java.util.ServiceLoader;

/**
 * @author Matthew Hua
 * @Date 2023/6/9
 */
public class MainTest {

    public static void main(String[] args) throws ClassNotFoundException, NoSuchMethodException, InvocationTargetException, InstantiationException, IllegalAccessException {
        Class<Player> aClass = (Class<Player>) Class.forName("io.matt.oop.superDemo.SpiderMan");
        Object o = aClass.getDeclaredConstructor().newInstance();
    }
}
