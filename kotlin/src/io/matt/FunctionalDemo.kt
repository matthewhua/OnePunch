package io.matt

/**
 * 它们的唯⼀⽬的是在对象的上下⽂中执⾏代码块。 当对⼀
个对象调⽤这样的函数并提供⼀个 lambda 表达式时，它会形成⼀个临时作⽤域。在此
作⽤域中，可以访问该对象⽽⽆需其名称。这些函数称为作⽤域函数。 共有以下五
种： let 、 run 、 with 、 apply 以及 also 。
 */

data class Person(var name: String, var age: Int, var city: String) {
    fun moveTo(newCity: String) { city = newCity }
    fun incrementAge() { age++ }
}

fun letDemo(){
    //如果不使⽤ let 来写这段代码，就必须引⼊⼀个新变量，并在每次使⽤它时重复其名称

    Person("Alice", 20, "Amsterdam").let {
        println(it)
        it.moveTo("London")
        it.incrementAge()
        println(it)
    }
}

fun main() {
    letDemo()
}

