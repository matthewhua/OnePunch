package io.matthew.interview;

import org.jetbrains.annotations.NotNull;

import java.util.concurrent.BlockingQueue;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;

/**
 *
 * ThreadPoolExecutor 的扩展主要是通过重写它的 beforeExecute() 和 afterExecute() 方法实现的，我
 * 们可以在扩展方法中添加日志或者实现数据统计，比如统计线程的执行时间，
 * @author: Matthew
 * @created 2022/12/16 15:18
 */
public class ThreadPoolExtend {


    public static void main(String[] args) {
        final MyThreadPoolExecutor executor = new MyThreadPoolExecutor(2, 4, 10, TimeUnit.SECONDS, new LinkedBlockingQueue<>());
        for (int i = 0; i < 4; i++) {
            executor.execute(() -> Thread.currentThread().getName());
        }
    }


    static class MyThreadPoolExecutor extends ThreadPoolExecutor {

        private final ThreadLocal<Long> localTime = new ThreadLocal<>();


        public MyThreadPoolExecutor(int corePoolSize, int maximumPoolSize, long keepAliveTime, @NotNull TimeUnit unit, @NotNull BlockingQueue<Runnable> workQueue) {
            super(corePoolSize, maximumPoolSize, keepAliveTime, unit, workQueue);
        }

        @Override
        protected void beforeExecute(Thread t, Runnable r) {
            final long sTime = System.nanoTime();
            localTime.set(sTime);
            System.out.printf("%s | before | time=%s%n", t.getName(), sTime);
        }

        @Override
        protected void afterExecute(Runnable r, Throwable t) {
            final long eTime = System.nanoTime();
            final long totalTime = eTime - localTime.get();
            System.out.printf("%s | after | time=%s | 耗时： %s 毫秒%n",
                    Thread.currentThread().getName(), eTime, (totalTime / 1000000.0));
        }
    }
}
