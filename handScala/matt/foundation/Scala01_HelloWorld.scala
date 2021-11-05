package handsome.matt.foundation



/**
 * @author Matthew
 * @date 2020-12-30 22:38
 */
object Scala01_HelloWorld {

 /*
 >Unit 相当于java的void
 * */
  def main(args: Array[String]): Unit = {
    println("Hello Scala")

  }

}

object Person{

}

//trait has its own main method
object Hello2 extends App{
  println("Hello World")
}

object twoTypesVar extends App{
  val ss = "hello" // 不可变
  var i = 52        // 可变

}

object Test extends App {
  var a = 0;
  val numList = List(1,2,3,4,5,6,7,8,9,10)

  // for 循环
  var retVal = for { a <- numList
                      if a != 3; if a < 8
                    }yield a;

  // 输出 返回值
  for (a <- retVal){
    println("Value of a:" + a)
  }

}

object TwoNotesAboutStrings extends App {
  val firstName = "John";
  val mi = 'C'
  val lastName = "Doe"
  // you can append them together like this, if you want to:(Java like this)
  val name = firstName + " " + mi + " " + lastName

  //Scala provides this more convenient form:
  val scName = s"$firstName $mi $lastName"

  // simple way to fix this problem is to put
  // a | symbol in front of all lines after the first line, and call the stripMargin method after the string:
  // 不然会有很大的空间隔阂
  val speech = """Four score and
                 |seven years ago
                 |our fathers ...""".stripMargin
  println(speech)
  println(scName)
}


