package io.matt.behavior.chainOfResponsibility.preCheck;

public class NegativeExecutor implements Executor {

    private Executor next;

    @Override
    public void setNext(Executor executor) {
        this.next = executor;
    }

    @Override
    public void handle(Integer num) {
        if (num != null && num < 0) {
            System.out.println("NegativeExecutor获取数字: " + num + ", 处理完成！");
        } else {
            if (next != null) {
                System.out.println("===经过NegativeExecutor");
                next.handle(num);
            } else {
                System.out.println("处理终止！ -NegativeExecutor");
            }
        }
    }
}
