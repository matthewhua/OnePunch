package io.matt.behavior.visitor.router;

public class TPLinkRouter implements Router {

    @Override
    public void sendData(char[] data) {

    }

    @Override
    public void accept(RouterVisitor v) {
        v.visit(this);
    }
}
