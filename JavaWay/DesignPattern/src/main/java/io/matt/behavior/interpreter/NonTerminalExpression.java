package io.matt.behavior.interpreter;

public class NonTerminalExpression implements AbstractExpression {

    AbstractExpression expression1;
    AbstractExpression expression2;

    public NonTerminalExpression(AbstractExpression expression1, AbstractExpression expression2) {
        this.expression1 = expression1;
        this.expression2 = expression2;
    }

    @Override
    public boolean interpreter(Context context) {
        return expression1.interpreter(context) && expression2.interpreter(context);
    }
}
