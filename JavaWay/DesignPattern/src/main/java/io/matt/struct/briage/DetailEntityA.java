package io.matt.struct.briage;

public class DetailEntityA extends AbstractEntity{

    public DetailEntityA(AbstractBehavior myBehavior) {
        super(myBehavior);
    }

    @Override
    public void request() {
        super.myBehavior.operationOne();
    }
}
