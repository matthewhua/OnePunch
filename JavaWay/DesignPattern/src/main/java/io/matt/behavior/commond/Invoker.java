package io.matt.behavior.commond;

public class Invoker {

    Command[] commands;

    public void setCommand(Command... command) {
        commands = command;
    }

    public void run() {
        for (Command command : commands) {
            command.execute();
        }
    }
}
