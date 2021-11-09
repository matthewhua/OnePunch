package matt.foundation.futures

import scala.concurrent.Future
import scala.util.{Failure, Success}
import scala.concurrent.ExecutionContext.Implicits.global
/**
 * @author Matthew
 * @date 2021/11/9 10:24
 */
object MultipleFutures {


  val startTime = currentTime

  // (a) create three futures
  val applFuture: Future[Double] = getStockPrice("APPL")
  val amznFuture = getStockPrice("AMZN")
  val googFuture = getStockPrice("GOOG")

  // (b) get a combined result in a for-comprehension
  private val result: Future[(Double, Double, Double)] = for {
    appl <- applFuture
    amzn <- amznFuture
    goog <- googFuture
  } yield (appl, amzn, goog)

  //(c) do whatever you need to do with the results
  result.onComplete{
    case Success(x) => {
      val endTime = deltaTime(startTime)
      println(s"In Success case, time delta: ${endTime}")
      println(s"The stock prices are: $x")
    }
    case Failure(exception) => exception.printStackTrace
  }

  // important for a little parallel demo: need to keep
  // the jvm’s main thread alive
  sleep(5000)

  def sleep(time: Long) = Thread.sleep(time)


  def getStockPrice(stockSymbol: String): Future[Double] = Future{
    val random = scala.util.Random
    val randomSleepTime = random.nextInt(3000)
    println(s"For $stockSymbol, sleep time is $randomSleepTime")
    val randomPrice = random.nextDouble * 1000
    sleep(randomSleepTime)
    randomPrice
  }

  def currentTime = System.currentTimeMillis()
  def deltaTime(t0: Long) = currentTime - t0

}
