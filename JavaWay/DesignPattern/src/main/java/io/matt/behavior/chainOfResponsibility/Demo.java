package io.matt.behavior.chainOfResponsibility;

public class Demo {

    public static void main(String[] args) {
        HandlerA handlerA = new HandlerA();
        HandlerB handlerB = new HandlerB();
        HandlerC handlerC = new HandlerC();
        handlerA.setNext(handlerB);
        handlerB.setNext(handlerC);

        Request request = new Request();
        request.setData("请求数据 ABCDE");
        handlerA.handle(request);
    }
}
