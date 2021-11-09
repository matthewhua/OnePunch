package matt.foundation

import scala.collection.mutable
import scala.collection.mutable.{ArrayBuffer, Map, Set};

/**
 * @author Matthew
 * @date 2021/11/5 16:58
 */
object CollectionDemo extends App
{
  val ints = ArrayBuffer[Int]()
  ints += 1
  ints += 2
  println(ints) // ArrayBuffer(1, 2)


  private val nums = ArrayBuffer(1, 2, 3)
  nums += 4 // add one element
  nums += 5 += 6 // // add multiple elements
  nums ++= List(7, 8, 9)  // add multiple elements from another collection
  nums -= 9 // remove one element
  nums -=7 -= 8 // remove multiple elements
  nums --= Array(5,6) // remove multiple elements using another collection
  println(nums) //1, 2, 3, 4, 5, 6, 7, 8, 9

  val a = ArrayBuffer(1, 2, 3)         // ArrayBuffer(1, 2, 3)
  a.append(4)                          // ArrayBuffer(1, 2, 3, 4)
  a.append(5, 6)                       // ArrayBuffer(1, 2, 3, 4, 5, 6)
  a.appendAll(Seq(7,8))                // ArrayBuffer(1, 2, 3, 4, 5, 6, 7, 8)
  a.clear

  val b = ArrayBuffer(9, 10)           // ArrayBuffer(9, 10)
  b.insert(0, 8)                       // ArrayBuffer(8, 9, 10)
  b.insertAll(0, Vector(4, 5, 6, 7))   // ArrayBuffer(4, 5, 6, 7, 8, 9, 10)
  b.prepend(3)                         // ArrayBuffer(3, 4, 5, 6, 7, 8, 9, 10)
  b.prepend(1, 2)                      // ArrayBuffer(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)
  b.prependAll(Array(0))


  private val ran: ArrayBuffer[Char] = ArrayBuffer.range('a', 'h') // ArrayBuffer(a, b, c, d, e, f, g)

  val names = ArrayBuffer[String]()
}

// The List class is a linear, immutable sequence, it’s a linked-list that you can’t modify.
object Lists extends App{
  val ints = List(1, 2, 3)
  val names = List("Joel", "Chris", "Ed")

  private val b = 0 +: ints
  private val c: List[Int] = List(-1, 0) ++: ints
  println(b) //List(0, 1, 2, 3)
  println(c) //List(-1, 0, 1, 2, 3)

  /**
   * LOOP the List
   */
  for (name <- names) println(name)

  // history
  //is very similar to the List class from the Lisp programming language
  val list = 1 :: 2 :: 3 :: Nil
  println(list) //List(1, 2, 3)
}

/**
 * 上面的
 * How to remember the method names
 *  but one way to remember those method names is to think that the : character represents the side that the sequence is on,
 *  so when you use +: you know that the list needs to be on the right,
 *  0 +: a
 *  Similarly, when you use :+ you know the list needs to be on the left:
 *  a :+ 4
 */


object VectorDemo extends App{
  private val nums = Vector(1, 2, 3)
  private val b: Any = nums :+ 4
  private val c: Vector[Int] = nums ++ Vector(4, 5) //添加整个Vector
  println(b)
  println(c)
  //You can also prepend elements like this:
  private val d: Vector[Int] = 0 +: nums   //右边加入
  println(d)
  private val e: Vector[Int] = Vector(-1, 0) ++: nums
  println(e)

  // loop over elements in a Vector just like you do with an ArrayBuffer or List:
  private val names = Vector("Joel", "Matthew", "Curtis")
  println(names)
  for (name <- names) println(name)

}

object MapDemo extends App{
  private var lover = mutable.Map("SHY" -> "Matthew")
  println(lover)

  // you can add a single element to the Map with +=
  lover += ("Vanida" -> "Matthew")
  //You also add multiple elements using +=: 擦,不推荐使用了
  lover += ("AR" -> "Arkansas", "AZ" -> "Arizona")
  //You can add elements from another Map using ++=:
  lover ++= Map("gloria" -> "Matthew", "Magritte" -> "Matthew")
  println(lover)

  //now , we're broke up
  lover -= "Vanida"
  lover -= ("AR", "AZ")
  lover --= List("gloria", "Magritte")
  println(lover)

  //You update Map elements by reassigning their key to a new value: put操作
  //lover("Matthew") = "SHY"  好家伙IDEA  知道没有Matthew(因为这是废弃掉的了.绝了
  // lover.updated("SHY", "Always is Matthew") // 3 才有
  private val str: String = lover("SHY")
  lover("SHY") = "Always is Matthew"
  println(lover)
}

object SetDemo extends App{
  private val set: mutable.Set[Int] = Set[Int]()
  //You add elements to a mutable Set with the +=, ++=, and add methods. Here are a few examples:
  set += 1
  set += 2 += 3
  set ++= Vector(4, 5)
  println(set)
  private val bool = set.add(6)
  println(bool)
  private val bool1: Boolean = set.add(5)
  println(bool1) //存在了, 不能添加了 false


  //delete the elements
  set -= 1
  println(set)
  //two or more elements (-= has a varargs field)
  set -= (2, 3)
  println(set)
  set --= Array(4, 5)
  println(set)

  // remove
  set.add(7)
  set.add(8)
  set.remove(7)
  println(set)
  set.clear()
  println(set)
}

object Anonymous extends App{
  //An anonymous function is like a little mini-function
  private val ints = List(1, 2, 3)
  private val doubledInts: List[Int] = ints.map(_ * 2)
  //_ * 2 This is a shorthand way of saying, “Multiply an element by 2.”
  //The _ character in Scala is something of a wildcard character. You’ll see it used in several different places.
  // In this case it’s a shorthand way of saying, “An element from the list, ints.”
  println(doubledInts) //List(2, 4, 6)
  // you can also write it like this
  private val tripleInts: List[Int] = ints.map((i: Int) => i * 3)
  println(tripleInts) //List(3, 6, 9)
  private val quadruple: List[Int] = ints.map(i => i * 4)
  println(quadruple);

  // Java Code:
/*  util.ArrayList<Integer> ints = new util.ArrayList<>(util.Arrays.asList(1, 2, 3));
  // the `map` process
  List<Integer> doubledInts = ints.stream()
    .map(i -> i * 2)
    .collect(Collectors.toList());*/

  //The map example shown is also the same as this Scala code:
  private val twoInts: List[Int] = for (i <- ints) yield i * 2

}

object AnonymousFunctions extends App{
  private val ints: List[Int] = List.range(1, 10) //the filter method of the List class
  //This is how you create a new list of all integers whose value is greater than 5:
  private val greatFive: List[Int] = ints.filter(_ > 5)
  println(greatFive)

  private val lessFive: List[Int] = ints.filter(_ < 5)
  println(lessFive)
  println( ints.drop(1)) // 并不会改变
  println(ints)

  //reduce
  // “map reduce,” the “reduce” part refers to methods like reduce.

  def add(x: Int, y: Int): Int = {
    val theSum = x + y
    println(s"received $x and $y, their sum is $theSum")
    theSum
  }
  println(lessFive.reduce(add)) //when you pass the add method into reduce:

  //you’ll write a “sum” algorithm like this:
  println(lessFive.reduce(_ + _))
  println(lessFive.reduce(_ * _))
  println(lessFive.reduce(_ / _))
  println(lessFive.reduce(_ - _))

}

object tupleDemo extends App{
  // aren’t collections classes, they’re just a convenient little container
  def getStockInfo = {
    ("NFLX", 100.00, 101.11) // this is a Tuple3
  }

  private val info: (String, Double, Double) = getStockInfo
  println(info)
}