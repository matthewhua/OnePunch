package io.matt

/**
 * @author Matthew
 * @date 2021-12-06 23:42
 */
class Shape

class Rectangle(var height:Double, var length:Double){
    var perimeter = (height + length) * 2
}

fun main()
{
    val rectangle = Rectangle(4.0, 3.0)
    println("The perimeter is ${rectangle.perimeter}")
}

//Inheritance between classes is declared by a colon (:). Classes are final by default;
// to make a class inheritable, mark it as open.

open class Shape2

class Rectangle2(var height:Double, var length:Double) : Shape2(){
    var perimeter = (height + length) * 2
}