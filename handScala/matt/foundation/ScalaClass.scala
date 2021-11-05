package handsome.matt.foundation

import scala.collection.mutable.ArrayBuffer

/**
 * @author Matthew
 * @date 2021/11/5 13:48
 */

object BasicClassConstruct extends App{

  class Person(var firstName: String, var lastName: String)
  {
    println("the constructor begins")

    // 'public ' access by default
    var age = 0

    // some class fields
    private val Home = System.getProperty("user.home")

    //some methods

    override def toString: String = s"$firstName $lastName is $age years old"
    def printHome(): Unit = println(s"HOME = $Home")
    def printFullName(): Unit = println(this)

    printHome()
    printFullName()
    println("you've reached the end of the constructor")
  }
  private val person = new Person("Matthew", "HUA")
  /*println(person)
  println(person.firstName + " " + person.lastName)*/
}

// 辅助的类构造器
object Auxiliary extends App{
    val DefaultCrustSize = 12
    val DefaultCrustType = "THIN"

    class Pizza(var crustSize: Int, var crustType: String){
        // one-arg auxiliary constructor
      def this(crustSize: Int) ={
        this(crustSize, DefaultCrustType)
      }

      // one-arg auxiliary constructor
      def this(crustType: String) = {
        this(DefaultCrustSize, crustType)
      }

      // zero-arg auxiliary constructor
      def this() = {
        this(DefaultCrustSize, DefaultCrustType)
      }

      override def toString: String = s"A $crustSize inch pizza with $crustType crust"
    }

  // follow this are same
  val p1 = new Pizza(DefaultCrustSize, DefaultCrustType)
  val p2 = new Pizza(DefaultCrustSize)
  val p3 = new Pizza(DefaultCrustType)
  val p4 = new Pizza
}

object supplying_default extends App{
/*
  class Socket(var timeout: Int, var linger:Int){
    override def toString = s"timeout: $timeout, linger: $linger"
  }
*/

  // better by supplying default values for the timeout and linger parameters:
  class Socket(var timeout: Int = 2000, var linger: Int = 3000){
    override def toString = s"timeout: $timeout, linger: $linger"
  }

  new Socket()
  new Socket(1000)
  new Socket(4000, 6000)

  private val s = new Socket(timeout = 2000, linger = 3000)
}


object method_first extends App{
  def double(a: Int) = a * 2
  //To show something a little more complex, here’s a method that takes two input parameters:

  def add(a: Int, b: Int): Int = a + b
  //Multiline methods

  def addThenDouble(a: Int, b: Int): Int = {
    val sum = a + b
    val doubled = sum * 2
    doubled
  }

  println(addThenDouble(2, 1))
}

/**
 * Enumerations
 */
sealed trait DayOfWeek
case object Sunday extends DayOfWeek
case object Monday extends DayOfWeek
case object Tuesday extends DayOfWeek
case object Wednesday extends DayOfWeek
case object Thursday extends DayOfWeek
case object Friday extends DayOfWeek
case object Saturday extends DayOfWeek

sealed trait Suit
case object Clubs extends Suit
case object Spades extends Suit
case object Diamonds extends Suit
case object Hearts extends Suit

sealed trait Topping
case object Cheese extends Topping
case object Pepperoni extends Topping
case object Sausage extends Topping
case object Mushrooms extends Topping
case object Onions extends Topping


sealed trait CrustSize
case object SmallCrustSize extends CrustSize
case object MediumCrustSize extends CrustSize
case object LargeCrustSize extends CrustSize

sealed trait CrustType
case object RegularCrustType extends CrustType
case object ThinCrustType extends CrustType
case object ThickCrustType extends CrustType

class Pizza(var crustSize:CrustSize = MediumCrustSize,
            var crustType: CrustType = RegularCrustType)
{

  // ArrayBuffer is a mutable sequence(list)
  private val toppings: ArrayBuffer[Topping] = scala.collection.mutable.ArrayBuffer[Topping]()
  def addTopping(t: Topping): Unit = toppings += t
  def removeTopping(t: Topping): Unit = toppings -= t
  def removeAllToppings(): Unit = toppings.clear()

  override def toString(): String = {
    s"""
       |Crust Size: $crustSize
       |Crust Type: $crustType
       |Toppings:   $toppings
        """.stripMargin
  }
}


object Enumerations extends App{
  private val pizza = new Pizza
  pizza.addTopping(Cheese)
  pizza.addTopping(Pepperoni)
  println(pizza)
}

