package io.matt;

import java.util.List;
import java.util.stream.Collectors;

public class JavaFilter {

    public void FilterOne()
    {
        // Java Filter 方式
        List<Integer> numbers = List.of(1, 2, 3, 4, 5, 6);
        String invertedOddNumbers = numbers
                .stream()
                .filter(it -> it % 2 != 0)
                .map(it -> -it)
                .map(Object::toString)
                .collect(Collectors.joining(", "));
        System.out.println(invertedOddNumbers);
    }
}
