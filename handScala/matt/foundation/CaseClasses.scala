package matt.foundation

import matt.foundation.CaseClasses.Person
import scala.reflect.io.File

/**
 * @author Matthew
 * @date 2021/11/8 16:47
 */

/**
 * A case class has all of the functionality of a regular class, and more. When the compiler sees the case keyword in front of a class, it generates code for you, with the following benefits:

Case class constructor parameters are public val fields by default, so accessor methods are generated for each parameter.
An apply method is created in the companion object of the class, so you don’t need to use the new keyword to create a new instance of the class.
An unapply method is generated, which lets you use case classes in more ways in match expressions.
A copy method is generated in the class. You may not use this feature in Scala/OOP code, but it’s used all the time in Scala/FP.
equals and hashCode methods are generated, which let you compare objects and easily use them as keys in maps.
A default toString method is generated, which is helpful for debugging.
 */
object CaseClasses extends App {
  case class Person(name: String, relation: String)

  private val christ: Person = Person("Christ", "niece") //apply is created
  println(christ) //Person(Christ,niece)
  println(christ.name)
  // christ.name = "Fred" //can't mutate the `name` field

  //Because in FP you never mutate data structures, it makes sense that constructor fields default to val.
}


trait Person {
  def name: String
}
// create these case classes to extend that trait:
case class Student(name: String, year: Int) extends Person
case class Teacher(name: String, specialty: String) extends Person

object unwrappedMethod extends App{
  def getPrintableString(p: Person): String = p match {
    case Student(name, year) =>
      s"$name is a student in Year $year."
    case Teacher(name, whatTheyTeach) =>
      s"$name teaches $whatTheyTeach."
  }

  //The Scala standard is that an unapply method returns the case class constructor fields in a tuple
  // that’s wrapped in an Option. The “tuple” part of the solution was shown in the previous lesson.
  val s = Student("Al", 1)
  val t = Teacher("Bob Donnan", "Mathematics")
  println(getPrintableString(s))
  println(getPrintableString(t))
}


/**
 * A case class also has an automatically-generated copy method
 */
object copyMethod extends App{
  case class BaseballTeam(name: String, lastWorldSeriesWin: Int)
  val cubs1908 = BaseballTeam("Chicago Cubs", 1908)
  val cubs2016 = cubs1908.copy(lastWorldSeriesWin = 2016)
  println(cubs1908)
  println(cubs2016)

  /**
   * Because you never mutate data structures in FP, this is how you create a new instance of a class from an existing instance.
   * This process can be referred to as, “update as you copy.”
   */
}

object equalsHashCode extends App{
  case class Person(name: String, relation: String)
  val christina = Person("Christina", "niece")
  private val hannah: Person = Person("Hannah", "niece")
  println(christina == hannah) //falase
}

/**
//A common example of this is when you create a “utilities” object, such as this one:
object PizzaUtils {
  def addTopping(p: Pizza, t: Topping): Pizza = ...
  def removeTopping(p: Pizza, t: Topping): Pizza = ...
  def removeAllToppings(p: Pizza): Pizza = ...
}

//or this
object FileUtils {
  def readTextFileAsString(filename: String): Try[String] = ...
  def copyFile(srcFile: File, destFile: File): Try[Boolean] = ...
  def readFileToByteArray(file: File): Try[Array[Byte]] = ...
  def readFileToString(file: File): Try[String] = ...
  def readFileToString(file: File, encoding: String): Try[String] = ...
  def readLines(file: File, encoding: String): Try[List[String]] = ...
}*/

/**
 * Case objects
A case object is like an object, but just like a case class has more features than a regular class, a case object has more features than a regular object. Its features include:

It’s serializable
It has a default hashCode implementation
It has an improved toString implementation
Because of these features, case objects are primarily used in two places (instead of regular objects):

When creating enumerations
When creating containers for “messages” that you want to pass between other objects (such as with the Akka actors library)
 */

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

case class Pizza (
                   crustSize: CrustSize,
                   crustType: CrustType,
                   toppings: Seq[Topping]
                 )




object objectDemo
{
  case class StartSpeakingMessage(textToSpeak: String)
  case object StopSpeakingMessage
  case object PauseSpeakingMessage
  case object ResumeSpeakingMessage
}


