package io.matt.behavior.chainOfResponsibility;

public class HandlerA implements Handler {

    private Handler next;

    public HandlerA() {
    }

    @Override
    public void setNext(Handler handler) {
        this.next = handler;
    }

    @Override
    public void handle(Request request) {
        System.out.println("HandlerA 执行代码逻辑, 处理:" + request.getData());
        request.setData(request.getData().replace("AB", ""));
        if (null != next) {
            next.handle(request);
        } else {
            System.out.println("执行终止！");
        }
    }


}
