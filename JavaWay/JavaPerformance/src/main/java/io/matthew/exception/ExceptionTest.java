package io.matthew.exception;


/**
 * 本测试用来测试Exception 性能
 *
 * Java 报错都会爆出来 Exception in thread "main" java.lang.RuntimeException: 你怎么回事
 * 	at io.matthew.entity.ExceptionTest.main(ExceptionTest.java:7)
 *
 *
 * Exception in thread "main" io.matthew.exception.MyException: 就是真的牛皮 (可以爆红，但没有堆栈信息）
 *
 */
public class ExceptionTest {

    public static void main(String[] args) {
        if (args.length == 0)
            throw new RuntimeException("你怎么回事");
        else if (args.length == 1)
            throw new MyException("就是真的牛皮");
        System.out.println("那是真的牛皮");
        //throw new MyException("你怎么回事");
    }
}
