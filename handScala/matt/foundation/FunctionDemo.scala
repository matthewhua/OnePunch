package matt.foundation


/**
 * @author Matthew
 * @date 2021/11/8 13:29
 */
object FunctionDemo extends App  {
  //foreach is impure is that it’s method signature declares that it returns the type Unit.

  // pure Function

  /**
   * A pure function is a function that depends only on its declared inputs and its internal algorithm to produce its output.
   * It does not read any other values from “the outside world” — the world outside of the function’s scope — and it does not modify any values in the outside world.
   * @param i
   * @return
   */
  def double(i: Int): Int = i * 2

  def sum(list: List[Int]): Int =   list match {
    case Nil => 0
    case head:: tail  => head + sum(tail)
  }

  private val v1: List[Int] = List.range(1, 4)
  println(sum(v1));

}

object NoNullVALUES extends App{
  //
  // Scala’s solution to this problem is to use a trio of classes known as Option, Some, and None. The Some and None classes are subclasses of Option
  /**
   * You declare that toInt returns an Option type
      If toInt receives a string it can convert to an Int, you wrap the Int inside of a Some
      If toInt receives a string it can’t convert, it returns a None
   */

  def toInt(s: String): Option[Int] = {
    try {
      Some(Integer.parseInt(s.trim))
    } catch {
      case e: Exception => None
    }
  }

  //Being a consumer of toInt
  toInt("1") match {
    case Some(i) => println(i)
    case None => println("That didn't work.")
  }
  //yield
  val stringA = "1"
  val stringB = "2"
  val stringC = "3"
  private val y: Option[Int] = for {
    a <- toInt(stringA)
    b <- toInt(stringB)
    c <- toInt(stringC)
  } yield a + b + c
  println(y) //Some(6)

  // foreach
  toInt("1").foreach(println)  //
  toInt("x").foreach(println) //it does nothing
  // None is just an empty container.

  class Address(
                 var street1: String,
                 var street2: Option[String],
                 var city: String,
                 var state: String,
                 var zip: String
               )

  private val santa = new Address(
    "1 Main Street",
    None,
    "North Pole",
    "Alaska",
    "12345"
  )

  val matthew = new Address(
    "123 Main Street",
    Some("Apt. 2B"),
    "Talkeetna",
    "Alaska",
    "99676"
  )


}

/**
 * This lesson was a little longer than the others, so here’s a quick review of the key points:

Functional programmers don’t use null values
A main replacement for null values is to use the Option/Some/None classes
Common ways to work with Option values are match and for expressions
Options can be thought of as containers of one item (Some) and no items (None)
You can also use Options when defining constructor parameters
 */

object COMPANION_OBJECTS extends App{
  //  伴生对象
  //First, a companion object and its class can access each other’s private members (fields and methods).
  class SomeClass  {
    def printFilename() = {
      println(SomeClass.HiddenFilename)
    }
    private val god = "Matthew"

    println( SomeClass.NewGod) //可以读到伴生对象的私有值

  }

  object SomeClass {
    private val HiddenFilename = "/tmp/foo.bar"
    private val NewGod = "Matthew"
  }

  private val clazz = new SomeClass
  println(clazz) // 输出Matthew

}

//Creating new instances without the new keyword
object COMPANION_apply extends App{
  class Person {
    var name: Option[String] = None
    var age: Option[Int] = None
    override def toString = s"$name, $age"
  }

  object Person {
    // a one-arg constructor
    def apply(name: Option[String]): Person = {
      var p = new Person
      p.name = name
      p
    }
    // a two-arg constructor
    def apply(name: Option[String], age: Option[Int]): Person = {
      var p = new Person
      p.name = name
      p.age = age
      p
    }

    def unapply(p: Person):String =  s"${p.name}, ${p.age}"

  }

  private val zenMasters = List(
    Person(Some("Nansen")),
    Person(None),
    Person(Some("JoShu")),
    Person(Some("Matthew"), Some(18)))

  zenMasters.foreach(println)
}

object COMPANION_unApply extends App{
  class Person(var name: String, var age: Int)

  object Person {
    def unapply(p: Person): String = s"${p.name}, ${p.age}"
  }
  val p = new Person("Lori", 29)

  val result = Person.unapply(p) // nothing to happened
  println(result)
  /**
   * when you put an unapply method in a companion object, it’s said that you’ve created an extractor method,
   * because you’ve created a way to extract the fields out of the object.
   */

}


object unApplyReturnDifferentType extends App{
  class Person(var name: String, var age: Int)

  object Person {
    def unapply(p: Person): Tuple2[String, Int] = (p.name, p.age)
  }
  val p = new Person("Lori", 29)
  private val result: (String, Int) = Person.unapply(p)
  println(result)
}

/**
 * The key points of this lesson are:

A companion object is an object that’s declared in the same file as a class, and has the same name as the class
A companion object and its class can access each other’s private members
A companion object’s apply method lets you create new instances of a class without using the new keyword
A companion object’s unapply method lets you de-construct an instance of a class into its individual components
 */
