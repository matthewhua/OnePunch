package io.matt.util;

import java.util.ArrayList;

/**
 * @author Matthew
 * @date 2022/5/6
 */
public class Lists {
    private Lists() {}

    /**
     * <E>是 泛型
     * ArrayList 是返回值
     * @return
     * @param <E>
     */
    public static <E> ArrayList<E> newArray() {
        return new ArrayList<E>();
    }

}
