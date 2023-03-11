package io.matt.struct.adpater;

public class Adapter extends TargetAbstraction {
    private OtherImpl other;

    public Adapter() {
        this.other = new OtherImpl();
    }

    @Override
    public String filter(String str) {
        other.preCheck(str);
        return other.replace(str);
    }
}

class OtherImpl {

    public OtherImpl() {
    }

    public String replace(String str) {
        return str.replaceAll("<", "[");
    }

    public void preCheck(String str) {
    }

}
