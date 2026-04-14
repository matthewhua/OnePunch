package io.matt

data class User2(val name:String, val id: Int)
{
    override fun equals(other: Any?) = (other is User2) && (other.id == this.id)
}


fun main()
{
    val user = User2("Alex", 1)
    println(user)

    val secondUser = User2("Alex", 1)
    val thirdUser = User2("Max", 2)

    println("user == secondUser: ${user == secondUser}")   // 4
    println("user == thirdUser: ${user == thirdUser}")


}

