package matt.foundation

import scala.collection.mutable.ArrayBuffer

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
  private val nums = Vector(1, 2, 3, 4, 5)
  private val b: Any = nums :+ 4
  private val value = Vector(4, 5)
  nums ++ value
}