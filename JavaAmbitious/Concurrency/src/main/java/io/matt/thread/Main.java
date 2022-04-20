package io.matt.thread;

public class Main {

    public static void main(String[] args) {
        //startMyThread();

        System.out.println(Thread.currentThread().getName());
        new Thread(new MyRunnable() {}).start();
    }



    private static void startMyThread() throws InterruptedException {
        MyThread myThread = new MyThread();
        myThread.start();
        myThread.join(); //暂停线程的执行，直到调用该方法的线程执行结束为止。可以使用该方法等待另一个Thread对象结束。

        System.out.println("main线程 - 执行完成");
    }
}
