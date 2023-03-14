package io.matt.behavior.interpreter;


// 终结符表达式类
public class TerminalExpression implements AbstractExpression {
    private String data;

    public TerminalExpression(String data) {
        this.data = data;
    }

    @Override
    public boolean interpreter(Context context) {
        return context.getData().contains(data);
    }
}
