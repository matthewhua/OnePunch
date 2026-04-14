package io.matt

/**
 * @author Matthew
 * @date 2021-12-06 23:47
 */
class StringTemplates {


}

fun  string(){
    var a = 1

    // simple name in template:
    val s1 = "a is $a"

    a = 2
    // arbitrary expression in template:
    val s2 = "${s1.replace("is", "was")}, but now is $a"
    print(s2)
}

fun main()
{
    string()
}

