package io.matt

import io.matt.Resource.name
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

class Rectangle3(){
    var length = 0
    var breadth = 0
    var color = 0xFAFAFA
}


/**
 *  配置对象的属性（apply）
 * 这对于配置未出现在对象构造函数中的属性⾮常有⽤。
 */
fun applySomething()
{

    Rectangle3().apply {
        length = 4
        breadth = 5
        color = 0xFAFAFA
    }
}

// Java 7's try-with-resources
fun autoClose(){
    // 等价于7
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


/**
 * 需要泛型信息的泛型函数
 */
public final class Gson{

/*    public <T>

    inline fun <reified T: Any> Gson.f*/
}

fun loopWithFlag() {
    val listOf = listOf(1, 2, 3, 4, 5, 6)
    listOf.forEach {
        if (it % 2 != 0) {
            return@forEach print("hello") //这里也会被执行
        }
        println(it)
    }
}

fun main(){
    creteDto()
    foo()
   /* val filter = JavaFilter()
    filter.FilterOne()*/
    println(name)
    swapTwoVariables()
    println("---------------------- test flag -----------------------------")
    loopWithFlag()
}