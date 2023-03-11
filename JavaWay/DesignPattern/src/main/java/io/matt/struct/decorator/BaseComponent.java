package io.matt.struct.decorator;


//具体组件
public class BaseComponent implements Component{
    @Override
    public void execute() {
        System.out.println("I am doing Something");
    }
}
