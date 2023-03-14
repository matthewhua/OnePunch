package io.matt.behavior.interpreter;

public class Demo {

    public static void main(String[] args) {
        TerminalExpression mick = new TerminalExpression("mick");
        TerminalExpression shy = new TerminalExpression("SHY");
        NonTerminalExpression isSingle = new NonTerminalExpression(mick, shy);
        Context con1 = new Context("mick, SHY");
        Context context2 = new Context("SHY,mock");
        Context context3 = new Context("spike");
        System.out.println(isSingle.interpreter(con1));
        System.out.println(isSingle.interpreter(context2));
        System.out.println(isSingle.interpreter(context3));
    }
}
