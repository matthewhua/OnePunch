package io.matt.behavior.chainOfResponsibility;

public class Request {
    private String data;

    public Request() {
    }

    public Request(String data) {
        this.data = data;
    }

    public String getData() {
        return data;
    }

    public void setData(String data) {
        this.data = data;
    }
}
