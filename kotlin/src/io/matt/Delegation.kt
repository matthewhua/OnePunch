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

fun main()
{
    val tomAraya = TomAraya("Thrash Metal")
    tomAraya.makeSound()
    val elvisPresley = ElvisPresley("Dancin' to the Jailhouse Rock.")
    elvisPresley.makeSound() //I'm The King of Rock 'N' Roll: Dancin' to the Jailhouse Rock.

}