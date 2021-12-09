package io.matt

import java.math.BigDecimal
import java.nio.file.Files
import java.nio.file.Paths


// provides a Customer class with the following functionality:
data class Customer(val name: String, val email: String)

/**
 * getters (and setters in case of var s) for all properties

equals()

hashCode()

toString()

copy()

component1(), component2(), ..., for all properties (see Data classes)
 */

fun creteDto() {
    val customer = Customer("matthew", "1229926359@qq.com")
    println(customer.name)
    println(customer.email)
    val copy2 = customer.copy()
    println(copy2)
}

//Default values for function parameters
fun foo(a: Int = 1, b: String = " "){
    println("a: $a , b : $b")
}

fun kotlinFilter()
{
    val nums = listOf(1, 2, 3, 4, 5, 6, 7)
    nums.filter { it % 2 != 0 }
        .joinToString { "${-it}" }
}

// 创建单例
object Resource{
    const val name = "Matthew"
}


// Java 7's try-with-resources
fun autoClose(){
    val stream = Files.newInputStream(Paths.get("/some/file.txt"))
    stream.buffered().reader().use { reader ->
        println(reader.readText())
    }
}


// 交换
fun swapTwoVariables(){
    var  a = 1
    var  b = 2
    a = b.also { b = a }
    println("Swap two Variables a: $a, b: $b") //a: 2, b: 1
}


// Mark code as incomplete (TODO)
fun calcTexts(): BigDecimal = TODO("Waiting for feedback from accounting ")


fun main(){
    creteDto()
    foo()
    val filter = JavaFilter()
    filter.FilterOne()
    println(Resource.name)
    swapTwoVariables()
}