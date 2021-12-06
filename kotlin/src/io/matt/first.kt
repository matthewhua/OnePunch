package io.matt

fun main()
{
    println("hello world")
    println(sum(1, 2))
    println(sum2(1, 2))
    printSum(1, 2)
    printSumWithoutReturn(1, 2)
    variable()
}

//A function with two Int parameters and Int return type.
fun sum(a: Int, b: Int): Int{
    return a + b;
}

fun sum2(a: Int, b: Int) = a + b;


// A function that returns no meaningful value.
fun printSum(a: Int, b: Int): Unit{
    println("sum of $a and $b is ${a + b}")
}


fun printSumWithoutReturn(a: Int, b: Int){
    println("sum of $a and $b is ${a + b}")
}

fun variable(){
    val a: Int = 1
    val b = a
    val c: Int  // Type required when no initializer is provided
    c = 3
    println("a $a b $b c $c")
}

