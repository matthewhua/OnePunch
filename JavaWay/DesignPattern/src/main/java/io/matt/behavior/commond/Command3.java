package io.matt.behavior.commond;

public class Command3 implements Command {

    private final Receiver receiver;

    public Command3(Receiver receiver) {
        this.receiver = receiver;
    }

    @Override
    public void execute() {
        receiver.operationC();
    }
}
