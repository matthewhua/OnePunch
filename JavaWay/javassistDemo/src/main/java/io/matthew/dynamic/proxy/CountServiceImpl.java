package io.matthew.dynamic.proxy;

/**
 * @author Matthew
 */
public class CountServiceImpl implements CountService{

    private int count = 0;

    @Override
    public int count() {
        return count;
    }
}
