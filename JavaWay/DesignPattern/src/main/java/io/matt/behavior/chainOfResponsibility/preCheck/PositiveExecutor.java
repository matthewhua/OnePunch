package io.matt.behavior.chainOfResponsibility.preCheck;

public class PositiveExecutor implements Executor {

    private Executor next;

    @Override
    public void setNext(Executor executor) {
        this.next = executor;
    }

    @Override
    public void handle(Integer num) {
        if (null != num && num > 0) {
            System.out.println("PositiveExecutor获取数字：" + num + " ,处理完成！");
        } else {
            if (next != null) {
                System.out.println("=== 经过PositiveExecutor");
                next.handle(num);
            } else {
                System.out.println("处理中止！-PositiveExecutor");
            }
        }

    }
}
