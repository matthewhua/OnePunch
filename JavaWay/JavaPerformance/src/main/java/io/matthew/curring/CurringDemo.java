package io.matthew.curring;

import java.util.function.BiFunction;

/**
 * @author: Matthew
 * @created 2022/10/26 18:42
 */
public class CurringDemo {

    public Letter createLetter(String salutation, String body) {
        return new Letter(salutation, body);
    }

    BiFunction<String, String, Letter> SIMPLE_LETTER_CREATOR
            = (salutation, body) -> new Letter(salutation, body);

}
