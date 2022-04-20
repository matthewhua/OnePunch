package io.matt.thread;


/**
 * 自己的线程
 * */
public class MyThread extends Thread {

    @Override
    public void run() {
        for (int i = 0; i < 10; i++) {
            System.out.println("MyThread线程：" + i);
        }
    }
}
