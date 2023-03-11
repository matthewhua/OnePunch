package io.matt.struct.adpater;

public class TargetAbstractionImpl extends TargetAbstraction{

    @Override
    public String filter(String str) {
        return str.replaceAll("a", "A");
    }
}
