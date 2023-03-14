package io.matt.behavior.chainOfResponsibility;

public class HandlerB implements Handler {

    private Handler next;

    public HandlerB() {
    }

    @Override
    public void setNext(Handler handler) {
        this.next = handler;
    }

    @Override
    public void handle(Request request) {
        System.out.println("HandlerB 执行代码逻辑, 处理:" + request.getData());
        request.setData(request.getData().replace("CD", ""));
        if (null != next) {
            next.handle(request);
        } else {
            System.out.println("执行终止！");
        }
    }
}
