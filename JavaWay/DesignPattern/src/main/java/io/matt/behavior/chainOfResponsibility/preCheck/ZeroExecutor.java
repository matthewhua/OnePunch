package io.matt.behavior.chainOfResponsibility.preCheck;

public class ZeroExecutor implements Executor {

    private Executor next;

    @Override
    public void setNext(Executor executor) {
        this.next = executor;
    }

    @Override
    public void handle(Integer num) {
        if (null != num && num == 0) {
            System.out.println("ZeroExecutor获取数字：" + num + " ,处理完成！");
        } else {
            if (null != next) {
                System.out.println("===经过ZeroExecutor");
                next.handle(num);
            } else {
                System.out.println("处理中止！-ZeroExecutor");
            }
        }
    }
}
