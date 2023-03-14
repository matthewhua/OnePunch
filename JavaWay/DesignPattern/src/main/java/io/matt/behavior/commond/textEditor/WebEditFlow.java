package io.matt.behavior.commond.textEditor;

import java.util.ArrayList;
import java.util.List;

public class WebEditFlow {

    private final List<Command> commands;

    public WebEditFlow() {
        this.commands = new ArrayList<>();
    }

    public void addCommand(Command command) {
        commands.add(command);
    }

    public void run() {
        commands.forEach(Command::execute);
    }
}
