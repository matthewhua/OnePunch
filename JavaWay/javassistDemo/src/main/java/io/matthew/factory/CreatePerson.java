package io.matthew.factory;

import javassist.*;

public class CreatePerson {

    /**
     * 创建一个Person 对象
     *
     * @throws Exception
     */
    public static void createPerson() throws Exception {
        final ClassPool pool = ClassPool.getDefault();

        // 1. 创建一个空类
        CtClass cc = pool.makeClass("io.matthew.entity.Person");

        // 2. 新增一个字段 private String name;
        // 字段名为name
        CtField param = new CtField(pool.get("java.lang.String"), "name", cc);
        // 访问级别 为private
        param.setModifiers(Modifier.PRIVATE);
        // 初始值是 "Matthew"
        cc.addField(param, CtField.Initializer.constant("Matthew"));

        // 3. 生成 getter, setter 方法
        cc.addMethod(CtNewMethod.setter("setName", param));
        cc.addMethod(CtNewMethod.getter("getName", param));

        //4. 添加无参的构造函数
        CtConstructor cons = new CtConstructor(new CtClass[]{}, cc);
        cons.setBody("{name = \" vanida \";}");
        cc.addConstructor(cons);

        // 5. 添加有参的构造函数
        cons = new CtConstructor(new CtClass[]{pool.get("java.lang.String")}, cc);
        // $0=this / $1,$2,$3... 代表方法参数
        cons.setBody("{$0.name = $1;}");
        cc.addConstructor(cons);

        // 6. 创建一个名为 printName方法, 无参数, 无返回值, 输出name 值
        final CtMethod ctMethod = new CtMethod(CtClass.voidType, "printName", new CtClass[]{}, cc);
        ctMethod.setModifiers(Modifier.PUBLIC);
        ctMethod.setBody("{System.out.println(name);}");
        cc.addMethod(ctMethod);

        //这里 会将这个创建的类对象编译为 .class 文件
        cc.writeFile("H:\\Dev\\GithubSource\\java\\OnePunch\\JavaWay\\javassistDemo\\src\\main\\java");
    }


    public static void main(String[] args) {
        try {
            createPerson();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
