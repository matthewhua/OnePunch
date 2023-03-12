package io.matt.behavior.visitor.router;

public interface RouterVisitor {

    void visit(DLinkRouter router);

    void visit(TPLinkRouter router);

    void visit(ASAURouter router);
}
