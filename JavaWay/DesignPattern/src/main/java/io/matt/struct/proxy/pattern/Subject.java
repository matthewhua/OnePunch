package io.matt.struct.proxy.pattern;


import java.util.Calendar;

// 抽象主题角色
public interface Subject {

    void method();
}


class RealSubject implements Subject {

    @Override
    public void method() {
        System.out.println("调用业务方法");
    }
}

class Log {
    public String getTime() {
        Calendar calendar = Calendar.getInstance();
        return calendar.getTime().toString();
    }
}

// 原版扩展代理
class ExProxy extends RealSubject {

    Log log;

    public ExProxy() {
        log = new Log();
    }

    void preCallMethod() {
        System.out.printf("方法method()被调用，调用时间为%s\n", log.getTime());
    }

    void postCallMethod(){
        System.out.printf("方法method()调用调用成功!\n");
    }

    @Override
    public void method() {
        preCallMethod();
        super.method();
        postCallMethod();
    }
}

class Proxy implements Subject {
    Log log;
    RealSubject realSubject;


    public Proxy() {
        realSubject = new RealSubject();
        log = new Log();
    }

    void preCallMethod() {
        System.out.printf("方法method()被调用，调用时间为%s\n", log.getTime());
    }

    void postCallMethod(){
        System.out.print("方法method()调用调用成功!\n");
    }

    @Override
    public void method() {
        preCallMethod();
        realSubject.method();
        postCallMethod();
    }
}