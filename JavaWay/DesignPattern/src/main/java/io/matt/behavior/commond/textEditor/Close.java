package io.matt.behavior.commond.textEditor;

public class Close implements Command {

    private Editor editor;

    public Close(Editor editor) {
        this.editor = editor;
    }

    @Override
    public void execute() {
        editor.close();
    }
}
