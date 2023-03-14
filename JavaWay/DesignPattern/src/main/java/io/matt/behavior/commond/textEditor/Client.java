package io.matt.behavior.commond.textEditor;

public class Client {

    public static void main(String[] args) {
        HtmlEditor htmlEditor = new HtmlEditor();
        MarkDownEditor markDownEditor = new MarkDownEditor();
        Open htmlOpen = new Open(htmlEditor);
        Save htmlSave = new Save(htmlEditor);
        Close htmlClose = new Close(htmlEditor);
        Open markOpen = new Open(markDownEditor);
        Save markSave = new Save(markDownEditor);
        Close markClose = new Close(markDownEditor);
        WebEditFlow webEditFlow = new WebEditFlow();
        webEditFlow.addCommand(htmlOpen);
        webEditFlow.addCommand(htmlSave);
        webEditFlow.addCommand(htmlClose);
        webEditFlow.addCommand(markOpen);
        webEditFlow.addCommand(markSave);
        webEditFlow.addCommand(markClose);
        webEditFlow.run();
    }
}
