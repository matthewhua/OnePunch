package io.matt.callable;

import java.util.concurrent.*;

public class Test2 {

    public static void main(String[] args) throws ExecutionException, InterruptedException {
        MyCallable myCallable = new MyCallable();

        ThreadPoolExecutor executor = new ThreadPoolExecutor(
                5, 5,
                1, TimeUnit.SECONDS,
                new ArrayBlockingQueue<>(10)
        ) {
            @Override
            protected void afterExecute(Runnable r, Throwable t) {
                // 如果在call方法执行过程中有错误，则可以在此处进行处理
                System.out.println("任务执行完毕" + t);
            }
        };

        Future<String> future = executor.submit(myCallable);
        String result = future.get();
        System.out.println(result);

        executor.shutdown();
    }
}
