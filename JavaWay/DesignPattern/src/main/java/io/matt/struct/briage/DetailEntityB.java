package io.matt.struct.briage;

public class DetailEntityB extends AbstractEntity {

    public DetailEntityB(AbstractBehavior myBehavior) {
        super(myBehavior);
    }

    @Override
    public void request() {
        super.myBehavior.operationTwo();
    }
}
