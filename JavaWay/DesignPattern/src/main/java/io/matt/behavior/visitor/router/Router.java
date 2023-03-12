package io.matt.behavior.visitor.router;

public interface Router {
    void sendData(char[] data);

    void accept(RouterVisitor v);
}
