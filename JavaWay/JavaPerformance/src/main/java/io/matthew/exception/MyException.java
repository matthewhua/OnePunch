package io.matthew.exception;

public class MyException extends RuntimeException {
    public MyException(String message) {
        super(message);
    }

    @Override
    public synchronized Throwable fillInStackTrace() {
        return this;
    }
}
