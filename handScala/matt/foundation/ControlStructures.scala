package matt.foundation



import java.io.FileNotFoundException
import scala.io.StdIn.readLine

/**
 * @author Matthew
 * @date 2021/11/5 11:31
 */
object HelloInteractive extends App {
    println("Enter your first name")
    val firstName = readLine()
    print("Enter your last name: ")
    val lastName = readLine()

    println(s"Your name is $firstName $lastName")
}

/**
 *  forLoop
 */

object ForLoopTest extends App{
   val num = Seq(1, 2, 3)
  for (n <- num) println(n)

  val people = List(
    "Bill",
    "Candy",
    "Karen",
    "Leo",
    "Regina"
  )

  //for (p <- people) println(p) //普通For 循环
  people.foreach(println)  // forEach
  //在 for 循环 中你可以使用分号 (;) 来设置多个区间，它将迭代给定区间所有的可能值。
  var a = 0;
  var b = 0;
  for (a <- 1 to 3; b <- 1 to 3){
    println("Value of a: " + a)
    println("Value of b: " + b)
  }
  var god = "Matthew"
  // for循环过滤
  for (p <- people if p == god) println(p)
}

object MapTest extends App{
  val ratings = Map(
    "Lady in the Water"  -> 3.0,
    "Snakes on a Plane"  -> 4.0,
    "You, Me and Dupree" -> 3.5
  )

  for ((name, rating) <- ratings)
    println(s"Movie:$name, Rating:$rating")

  // for Each
  ratings.foreach{
    case (movie, rating) => println(s"Movie:$movie, Rating:$rating")
  }
}

object forExpressions extends App {
  val  nums = Seq(1,2,3)
  private val doubledNum: Seq[Int] = for (n <- nums) yield n * 2
  println(doubledNum) //还可以输出类型

  //Capitalizing a list of strings // 大写, 资本化
  val names = List("adam", "david", "frank")
  private val ucNames: List[String] = for (name <- names) yield name.capitalize
  println(ucNames)

  //Using yield after for is the “secret sauce” that says,
  // “I want to yield a new collection from the existing collection that I’m iterating over in the for-expression, using the algorithm shown.”
  val names2 = List("_adam", "_david", "_frank")

  //I first need to remove the underscore character
  private val capNames: List[String] = for (name <- names2) yield {
    val nameWithoutUnderScore = name.drop(1)
    val capName = nameWithoutUnderScore.capitalize
    capName
  }

  println(capNames) // 变化了
  println(names2)   //没有变化

  //shortVersion

  private val capNames2 = for (name <- names2) yield {
    name.drop(1).capitalize
  }
  println(capNames2)
}


object matchExpress extends App{
  val i = 1
  i match {
    case 1  => println("January")
    case 2  => println("February")
    case 3  => println("March")
    case 4  => println("April")
    case 5  => println("May")
    case 6  => println("June")
    case 7  => println("July")
    case 8  => println("August")
    case 9  => println("September")
    case 10 => println("October")
    case 11 => println("November")
    case 12 => println("December")
    // catch the default with a variable so you can print it
    case _  => println("Invalid month")
  }
  val monthName = i match {
    case 1  => "January"
    case 2  => "February"
    case 3  => "March"
    case 4  => "April"
    case 5  => "May"
    case 6  => "June"
    case 7  => "July"
    case 8  => "August"
    case 9  => "September"
    case 10 => "October"
    case 11 => "November"
    case 12 => "December"
    case _  => "Invalid month"
  }

}


object method extends App{

  def convertBooleanToStringMessage(bool: Boolean): String = {
    if (bool) "true" else "false"
  }

  def convertBooleanToStringMessageOnMatch(bool: Boolean) : String = bool match {
    case true => "you said true"
    case false => "son of bitch"
  }

  def isTrue(a: Any) = a match {
    case 0 | "" => false
    case _ => true
  }

  val i = 1
  val evenOrOdd = i match {
    case 1 | 3 | 5 | 7 | 9 => println("odd")
    case 2 | 4 | 6 | 8 | 10 => println("even")
    case _ => println("some other number")
  }
  val count = 3
  count match {
    case 1 => println("one, a lonely number")
    case x if x == 2 || x == 3 => println("two's company, three's a crowd")
    case x if x > 3 => println("4+, that's a party")
    case _ => println("i'm guessing your number is zero or less")
  }

  i match {
    case a if 0 to 9 contains a => println("0-9 range: " + a)
    case b if 10 to 19 contains b => println("10-19 range: " + b)
    case c if 20 to 29 contains c => println("20-29 range: " + c)
    case _ => println("Hmmm...")
  }

  stock match {
    case x if (x.symbol == "XYZ" && x.price < 20) => println("buy")
    case x if (x.symbol == "XYZ" && x.price > 50) => println("Sell")
  }

  println( convertBooleanToStringMessage(true))
  println( convertBooleanToStringMessage(false))
  println( convertBooleanToStringMessageOnMatch(true))
  println( convertBooleanToStringMessageOnMatch(false))
}

object stock{
  var symbol = "XYZ"
  var price = 70
}

object try_catch_example extends App {

  try {
    val x = 1
  } catch {
    case foo: FileNotFoundException  => println("Couldn't find that file.")
  } finally {}

}