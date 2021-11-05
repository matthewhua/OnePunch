package matt.foundation
// cala trait is like the original Java interface,
// where you define the desired interface for some piece of functionality, but you don’t implement any behavior.
trait TailWagger{
  def startTail(): Unit
  def stopTail(): Unit
}

trait Speaker {
  def speak(): String
}


trait Runner {
  def startRunning(): Unit
  def stopRunning(): Unit
}

//  a class that extends the trait and implements those methods like this:
class Dog extends TailWagger  {
  //the implemented methods

  override def startTail(): Unit = println("tail is wagging")

  override def stopTail(): Unit = println("tail is stopped")

 /* def startTail() = println("tail is wagging")
  def stopTail() = println("tail is stopped")*/
}

class MultiDog extends Speaker with TailWagger with Runner{
  // Speaker
  def speak(): String = "Woof!"

  override def startTail(): Unit = println("tail is wagging")

  override def stopTail(): Unit = println("tail is stopped")

  override def startRunning(): Unit = println("I'm running")

  override def stopRunning(): Unit = println("Stopped running")

}


object DogTest extends App{
  private val dog = new Dog
  dog.startTail()
}


// 子类重写父类 Mixing in multiple traits that have behaviors
object Mixing extends App{
  trait Speak{
    def speak(): String //abstract
  }
  trait TailWagger {
    def startTail(): Unit = println("tail is wagging")
    def stopTail(): Unit = println("tail is stopped")
  }

  trait Runner {
    def startRunning(): Unit = println("I'm running")
    def stopRunning(): Unit = println("Stopped running")
  }

  class Dog(name: String) extends Speak with TailWagger with Runner{
    def speak(): String = "Woof!"
  }

  class Cat extends Speak with TailWagger with Runner{
    def speak(): String ="Meow!"

    override def startRunning(): Unit = println("Yeah ... I don't run")

    override def stopRunning(): Unit = println("No need to stop")
  }

  private val bill = new Dog("Bill")
  bill.speak()
  bill.startTail()
  bill.startRunning()

  private val cat = new Cat
  cat.speak()
}

object AbstractClass extends App{
  abstract class Pet(name: String){
    def speak(): Unit = println("Yo")
    def comeToMaster() = println("Here I come !!!")
  }

  class Dog(name: String) extends Pet(name) {
    override def speak() = println("Woof")
    override def comeToMaster() = println("Here I come!")
  }

  private val d = new Dog("Rover")
  d.speak()
  d.comeToMaster()
}
