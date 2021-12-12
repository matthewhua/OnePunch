package io.matt

import kotlin.reflect.KProperty


class Example{
    var p: String by Delegate()

    override fun toString() =   "Example Class"
}

class Delegate() {
    operator fun getValue(thisRef: Any?, prop: KProperty<*>): String {
        return "$thisRef, thank you for delegating '${prop.name}' to me! "
    }

    operator fun setValue(thisRef: Any?, prop: KProperty<*>, value: String) {
         println("$value has been assigned to ${prop.name} in this $thisRef")
    }
}


/********************************* Standard Delegation ************************/
class LazySample{
    init {
        println("create ")
    }

    val  lazyStr: String by lazy {
        println("computed!!")
        "matt lazy"
    }
}


class User(val map: Map<String, Any?>) {
    val name: String by map                // 1
    val age: Int     by map                // 1
}



fun main()
{
    val example = Example()
    println(example.p)
    example.p = "NEW" //* 后面会触发 NEW has been assigned to p in this Example Class

    val lazySample = LazySample()
    println("lazyStr = ${lazySample.lazyStr}")
    println(" = ${lazySample.lazyStr}")


    val user = User(
        mapOf(
            "name" to "John Doe",
            "age" to 25
        )
    )

    println("name = ${user.name}, age = ${user.age}")
}


