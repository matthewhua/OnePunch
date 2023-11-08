package xyz.ariane.util.akka

import akka.actor.ActorRef
import akka.actor.ActorSystem
import akka.event.Logging
import akka.event.LoggingAdapter
import akka.http.javadsl.Http
import akka.http.javadsl.model.ContentType
import akka.http.javadsl.model.HttpRequest
import akka.http.javadsl.model.MediaTypes
import akka.http.javadsl.model.StatusCode
import xyz.ariane.util.akka.AkkaHttp.TIMEOUT_MILLIS
import xyz.ariane.util.dclog.lzError
import xyz.ariane.util.lang.NamedRunnable
import java.util.concurrent.CompletableFuture.completedFuture

interface HttpCorCbWrap {
    fun onSuccess(deal: suspend (StatusCode, String) -> Unit, statusCode: StatusCode, resp: String)

    fun onFailure(deal: suspend (StatusCode?, Throwable) -> Unit, statusCode: StatusCode?, t: Throwable)
}

/**
 * akka包装的一个HTTP对象。
 */
object AkkaHttp {

    lateinit var actorSystem: ActorSystem

    lateinit var http: Http

    lateinit var logger: LoggingAdapter

    const val TIMEOUT_MILLIS = 3000L

    val DEFAULT_CONTENT_TYPE: ContentType.WithFixedCharset =
        MediaTypes.APPLICATION_X_WWW_FORM_URLENCODED.toContentType()

    val APP_JSON: ContentType.WithFixedCharset = MediaTypes.APPLICATION_JSON.toContentType()

    /**
     * 初始化
     */
    fun initialize(system: ActorSystem) {
        actorSystem = system
        http = Http.get(system)
        logger = Logging.getLogger(system, javaClass)
    }

    /**
     * 执行Http GET请求，使用指定的actor执行回调
     *
     *
     * **注意：需要[handleResponseActor]接收并处理[Runnable]**
     */
    fun doGet(
        uri: String, handleResponseActor: ActorRef,
        handleResponse: (StatusCode, String) -> Unit,
        handleFailed: (StatusCode?, Throwable) -> Unit
    ) {
        http.doSingleRequest(HttpRequest.GET(uri),
            actorSystem,
            logger,
            { statusCode, resp ->
                handleResponseActor.tellNoSender(NamedRunnable("AkkaHttpDoGet") { handleResponse(statusCode, resp) })
            },
            { statusCode, throwable ->
                handleResponseActor.tellNoSender(NamedRunnable("AkkaHttpDoGet") { handleFailed(statusCode, throwable) })
            }
        )
    }

    /**
     * 使用[DEFAULT_CONTENT_TYPE]执行Http POST请求，使用指定的actor执行回调
     *
     * **注意：需要[handleResponseActor]接收并处理[Runnable]**
     */
    fun doPost(
        uri: String,
        body: String,
        handleResponseActor: ActorRef,
        onSuccess: (StatusCode, String) -> Unit,
        onFailure: (StatusCode?, Throwable) -> Unit
    ) {
        val rn = "AkkaHttpDoPost"
        val httpRequest = HttpRequest.POST(uri).withEntity(DEFAULT_CONTENT_TYPE, body)
        http.doSingleRequest(
            httpRequest,
            actorSystem,
            logger,
            { statusCode, resp ->
                handleResponseActor.tellNoSender(NamedRunnable(rn) { onSuccess(statusCode, resp) })
            },
            { status, err ->
                handleResponseActor.tellNoSender(NamedRunnable(rn) { onFailure(status, err) })
            }
        )
    }

    /**
     * 使用json执行HTTP POST请求
     */
    fun doPostUseJson(
        uri: String,
        body: String,
        handleResponseActor: ActorRef,
        onSuccess: (StatusCode, String) -> Unit,
        onFailure: (StatusCode?, Throwable) -> Unit
    ) {
        val rn = "AkkaHttpDoPost"
        val httpRequest = HttpRequest.POST(uri).withEntity(APP_JSON, body)
        http.doSingleRequest(
            httpRequest,
            actorSystem,
            logger,
            { statusCode, resp ->
                handleResponseActor.tellNoSender(NamedRunnable(rn) { onSuccess(statusCode, resp) })
            },
            { status, err ->
                handleResponseActor.tellNoSender(NamedRunnable(rn) { onFailure(status, err) })
            }
        )
    }

    /**
     * 执行Http GET请求，需要指定协程环境，在协程环境下处理结果
     *
     * @see [doSingleRequest]
     */
    fun doGet4Coroutine(
        url: String,
        wrap: HttpCorCbWrap,
        onSuccess: suspend (StatusCode, String) -> Unit,
        onFailure: suspend (StatusCode?, Throwable) -> Unit
    ) {
        val httpRequest = HttpRequest.GET(url)
        http.doSingleRequest(
            httpRequest,
            actorSystem,
            logger,
            { statusCode, resp -> wrap.onSuccess(onSuccess, statusCode, resp) },
            { status, err ->
                wrap.onFailure(onFailure, status, err)
            })
    }

    /**
     * 执行Http POST请求，需要指定协程环境，在协程环境下处理结果
     *
     * @see [doSingleRequest]
     */
    fun doPost4Coroutine(
        url: String,
        body: String,
        wrap: HttpCorCbWrap,
        onSuccess: suspend (StatusCode, String) -> Unit,
        onFailure: suspend (StatusCode?, Throwable) -> Unit
    ) {
        val httpRequest = HttpRequest.POST(url).withEntity(DEFAULT_CONTENT_TYPE, body)
        http.doSingleRequest(
            httpRequest,
            actorSystem,
            logger,
            { statusCode, resp -> wrap.onSuccess(onSuccess, statusCode, resp) },
            { status, err ->
                wrap.onFailure(onFailure, status, err)
            })
    }

    /**
     * 使用json执行HTTP POST请求，需要指定协程环境，在协程环境下处理结果
     */
    fun doPostUseJson4Coroutine(
        uri: String,
        body: String,
        wrap: HttpCorCbWrap,
        onSuccess: suspend (StatusCode, String) -> Unit,
        onFailure: suspend (StatusCode?, Throwable) -> Unit
    ) {
        val httpRequest = HttpRequest.POST(uri).withEntity(APP_JSON, body)
        http.doSingleRequest(
            httpRequest,
            actorSystem,
            logger,
            { statusCode, resp -> wrap.onSuccess(onSuccess, statusCode, resp) },
            { status, err ->
                wrap.onFailure(onFailure, status, err)
            })
    }

    /**
     * 执行Http GET请求，使用默认的线程池处理回调
     *
     * **小心：容易产生线程安全问题**
     *
     * FIXME add onFailure
     *
     * @see [doSingleRequest]
     */
    fun doGet_下面回调线程不安全_不能读写任何可变状态(url: String, handleResponse: (StatusCode, String) -> Unit) {
        val httpRequest = HttpRequest.GET(url)
        http.doSingleRequest(
            httpRequest,
            actorSystem,
            logger,
            onSuccess = { statusCode, resp -> handleResponse(statusCode, resp) })
    }

    fun doRequest_下面回调线程不安全_不能读写任何可变状态(request: HttpRequest, handleResponse: (StatusCode, String) -> Unit) {
        http.doSingleRequest(request, actorSystem, logger, handleResponse)
    }

}

/**
 * Http请求返回status code为失败类时，产生此异常
 */
class AkkaHttpFailureException(val statusCode: StatusCode?) : RuntimeException()

/**
 * 使用akka http发出请求，[onSuccess]和[onFailure]回调将会在ForkJoinPool的commonPool中调用
 *
 * **注意：由于commonPool默认最大线程数为1倍CPU核数，数量较少，因此[onSuccess]和[onFailure]不要做io或复杂计算等慢操作**
 *
 * @param request 请求
 * @param onSuccess 成功时的回调
 * @param onFailure 失败时的回调
 * @see AkkaHttpFailureException
 */
fun Http.doSingleRequest(
    request: HttpRequest,
    actorSystem: ActorSystem,
    logger: LoggingAdapter,
    onSuccess: (StatusCode, String) -> Unit,
    onFailure: (StatusCode?, Throwable) -> Unit = { _, _ -> Unit }
) {
    singleRequest(request)
        .thenCompose { response ->
            response.entity()
                .toStrict(TIMEOUT_MILLIS, actorSystem)
                .thenCombine(completedFuture(response.status())) { strict, status -> strict to status }
        }
        .whenComplete { pair, err ->
            if (err != null) {
                onFailure(null, err)
                logger.lzError { "Request error, msg=${err.message}\n  req=$request" }
            } else if (pair != null) {
                val (strict, status) = pair
                if (status.isSuccess) {
                    val s = strict.data.utf8String().trim()
                    onSuccess(status, s)
                } else {
                    onFailure(status, AkkaHttpFailureException(status))
                    logger.lzError { "Request failed, status=$status, reason=${status.reason()}, msg=${status.defaultMessage()}, req=$request" }
                }
            } else {
                onFailure(null, AkkaHttpFailureException(statusCode = null))
            }
        }
}

fun ActorRef.tellNoSender(msg: Any) {
    tell(msg, ActorRef.noSender())
}
