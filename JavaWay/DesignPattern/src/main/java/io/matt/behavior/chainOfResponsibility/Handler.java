package io.matt.behavior.chainOfResponsibility;

public interface Handler {

    void setNext(Handler handler);

    void handle(Request request);
}
