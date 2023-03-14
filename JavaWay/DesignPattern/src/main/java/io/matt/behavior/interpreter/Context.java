package io.matt.behavior.interpreter;

// 上下文信息类
public class Context {

    private String data;

    public Context(String data) {
        this.data = data;
    }

    public String getData() {
        return data;
    }

    public void setData(String data) {
        this.data = data;
    }
}
