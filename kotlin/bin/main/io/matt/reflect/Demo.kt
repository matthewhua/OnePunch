package io.matt.reflect

import io.matt.x


/**
 * 最基本的反射功能是获取 Kotlin 类的运⾏时引⽤。要获取对静态已知的 Kotlin 类的引
⽤，可以使⽤ 类字⾯值 语法：

函数、属性以及构造函数的引⽤，除了作为⾃省程序结构外， 还可以⽤于调⽤或者⽤作
函数类型的实例。
所有可调⽤引⽤的公共超类型是 KCallable<out R> ， 其中 R 是返回值类型，对于属
性是属性类型，对于构造函数是所构造类型。
 */


fun isOdd(x: Int) = x % 2 != 0

/**
 * 当上下⽂中已知函数期望的类型时， :: 可以⽤于重载函数。 例如：
 */
fun isOdd(s: String) = s == "brillig" || s == "slithy" || s == "tove"

/**
 *   // 如果需要使⽤类的成员函数或扩展函数，它需要是限定的，例如
 *   String::toCharArray 。
 *
 *   请注意，即使以扩展函数的引⽤初始化⼀个变量，其推断出的函数类型也会没有接收者
    （它会有⼀个接受接收者对象的额外参数）。如需改为带有接收者的函数类型，请明确
    指定其类型：
 */

 val isEmptyStringList: List<String>.() -> Boolean = List<String>::isEmpty

fun main(){
    val numbers = listOf(1, 2, 3)
    println(numbers.filter(::isOdd)) //这⾥ ::isOdd 是函数类型 (Int) -> Boolean 的⼀个值。

    //或者，你可以通过将⽅法引⽤存储在具有显式指定类型的变量中来提供必要的上下⽂
    val predicate: (String) -> Boolean = ::isOdd //引⽤到 isOdd(x: String)



}