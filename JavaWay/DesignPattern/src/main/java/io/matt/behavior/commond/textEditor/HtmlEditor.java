package io.matt.behavior.commond.textEditor;

public class HtmlEditor implements Editor {

    @Override
    public void open() {
        System.out.println("=== Html5Editor 执行 open 操作");
    }

    @Override
    public void save() {
        System.out.println("=== Html5Editor 执行 save 操作");
    }

    @Override
    public void close() {
        System.out.println("=== Html5Editor 执行 close 操作");
    }
}
