package io.matt.oop;

/**
 * @author Matthew
 * @date 2022/5/6
 */
public class SimpleExecutor implements Executor {

    @Override
    public void execute(Runnable runnable) {
        System.out.println("我来执行了 。。。");
    }
}
