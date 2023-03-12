package io.matt.behavior.visitor.router;

public class WindowsRouterVisitor implements RouterVisitor {

    @Override
    public void visit(DLinkRouter router) {
        System.out.println("=== DLinkRouter Windows visit success!");
    }

    @Override
    public void visit(TPLinkRouter router) {
        System.out.println("=== TPLinkRouter Windows visit success!");
    }

    @Override
    public void visit(ASAURouter router) {
        System.out.println("=== ASAURouter Windows visit success!");
    }
}
