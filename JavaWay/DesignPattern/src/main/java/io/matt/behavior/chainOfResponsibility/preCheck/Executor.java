package io.matt.behavior.chainOfResponsibility.preCheck;

public interface Executor {

    void setNext(Executor executor);

    void handle(Integer num);
}
