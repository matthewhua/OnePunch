package io.matt

import java.util.*

interface SoundBehavior{
    fun makeSound()
}

class ScreamBehavior(val n: String): SoundBehavior{
    override fun makeSound() =  println("${n.uppercase(Locale.getDefault())} !!!")
}


class RockAndRollBehavior(val n: String): SoundBehavior{
    override fun makeSound() =  println("I'm The King of Rock 'N' Roll: $n")
}

// Tom Araya is the "singer" of Slayer
class TomAraya(n:String): SoundBehavior by ScreamBehavior(n) //委托


// You should know ;)
class ElvisPresley(n: String): SoundBehavior by RockAndRollBehavior(n)


// The delegate object is defined after the by keyword. As you see, no boilerplate code is required.

private fun testDelegation1() {
    val tomAraya = TomAraya("Thrash Metal")
    tomAraya.makeSound()
    val elvisPresley = ElvisPresley("Dancin' to the Jailhouse Rock.")
    elvisPresley.makeSound() //I'm The King of Rock 'N' Roll: Dancin' to the Jailhouse Rock.
}

interface Base{
    fun print()
}

class BaseImpl(val x: Int): Base{
    override fun print() {
        println(x)
    }
}

class Derived(b: Base): Base by  b

fun testDelegation2() {
    val b = BaseImpl(10)
    Derived(b).print()
}

interface Base2{
    fun printMessage()
    fun printMessageLine()
}

class BaseImpl2(val x: Int) : Base2{
    override fun printMessage() {
        println(x)
    }

    override fun printMessageLine() {
        println(x)
    }
}

class Derived2(b: Base2): Base2 by b{
    override fun printMessage() { println("abc")}
}

fun testDelegation3() {
    val b = BaseImpl2(10)
    Derived2(b).printMessage()
    Derived2(b).printMessageLine()
}

//但请注意，以这种⽅式重写的成员不会在委托对象的成员中调⽤ ，委托对象的成员只能 访问其⾃身对接⼝成员实现：
interface Base3 {
    val message: String
    fun print()
}

class BaseImpl3(val x: Int) : Base3 {

    override val message = "BaseImpl: x = $x"

    override fun print() {
        println(message)
    }
}

class Derived3(b: Base3): Base3 by b{
    // 在 b 的 `print` 实现中不会访问到这个属性
    override val message = "Message of Derived"
}

fun testDelegation4(){
    val b = BaseImpl3(10)
    val derived = Derived3(b)
    derived.print()
    println(derived.message) //Message of Derived
}

fun main()
{
   /* testDelegation1()
    testDelegation2()
    testDelegation3()*/
    testDelegation4()
}

