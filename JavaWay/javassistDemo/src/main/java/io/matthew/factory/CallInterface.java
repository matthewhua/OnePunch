package io.matthew.factory;

import javassist.ClassPool;
import javassist.CtClass;

public class CallInterface {


    public static void  callName() throws Exception {
        final ClassPool pool = ClassPool.getDefault();
        pool.appendClassPath("D:\\develop\\practive\\OnePunch\\JavaWay\\javassistDemo\\src\\main\\java");

        //获取接口
        final CtClass codeClass = pool.get("io.matthew.factory.PersonI");
        //获取上面生成的类
        final CtClass ctClass = pool.get("io.matthew.entity.Person");
        // 使代码生成的类，实现 PersonI 接口
        ctClass.setInterfaces(new  CtClass[]{codeClass});


        // // 以下通过接口直接调用 强转
        final PersonI person = (PersonI) ctClass.toClass().newInstance();
        System.out.println(person.getName());
        person.setName("xiaoLKv");
        person.printName();
    }

    public static void main(String[] args) {
        try {
            callName();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
