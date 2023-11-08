@file:Suppress("EnumEntryName")

package xyz.ariane.util.hotscript

import akka.actor.ActorRef
import akka.actor.Props
import akka.actor.UntypedAbstractActor
import akka.event.Logging
import akka.event.LoggingAdapter
import java.io.File
import java.net.URLClassLoader
import java.util.*
import java.util.jar.JarFile

const val SCRIPT_DAEMON_ACTOR_NAME = "scriptDaemon"
const val SCRIPT_DAEMON_PATH = "/user/$SCRIPT_DAEMON_ACTOR_NAME"

const val SCRIPT_DAEMON_OK_RESPONSE = "OK"

const val PATCH_RUNNABLE_CLASS_ATTRIBUTE = "Patch-Runnable-Class"

// 脚本消息
interface ScriptMessage

enum class ScriptType { jar }

/**
 * 执行脚本的类
 */
open class ExecuteScript(val scriptData: ByteArray, val type: ScriptType) : ScriptMessage {
    constructor(another: ExecuteScript) : this(another.scriptData, another.type)
}

/**
 * 负责为所在进程提供脚本服务，每个节点启动一个
 *
 */
class ScriptDaemonActor : UntypedAbstractActor() {

    companion object {

        fun props(mailbox: String): Props =
                Props.create(ScriptDaemonActor::class.java) { ScriptDaemonActor() }
                        .withMailbox(mailbox)
                        .withDispatcher("akka.actor.scriptd-dispatcher")

    }

    private val logger: LoggingAdapter = Logging.getLogger(context.system(), javaClass)

    /** 已经加载过的jar补丁类名 */
    private val loadedJarPatchClassSet = hashSetOf<String>()

    override fun preStart() {
        logger.info("$self started.")
    }

    override fun postStop() {
        logger.info("$self stopped.")
    }

    override fun onReceive(msg: Any) {
        try {
            when (msg) {
                is ExecuteScript -> when (msg.type) {
                    ScriptType.jar -> handleExecuteJar(msg)
                }
            }
        } catch (e: Throwable) {
            logger.error(e, "handle message err: $msg")
        }
    }

    private fun handleExecuteJar(msg: ExecuteScript) {
        // 找到临时目录，在临时目录中创建临时jar。
        val tmpDir = System.getProperty("java.io.tmpdir") ?: "/tmp"
        val tmpJarFile = File("$tmpDir/patch-${UUID.randomUUID()}.jar")
        tmpJarFile.writeBytes(msg.scriptData)

        // 加载jar，判断类名是否用过了。
        val runnableClassName: String =
                JarFile(tmpJarFile).manifest.mainAttributes.getValue(PATCH_RUNNABLE_CLASS_ATTRIBUTE)
        if (runnableClassName in loadedJarPatchClassSet) {
            logger.error("Duplicate jar patch class $runnableClassName. Ignored.")
            return
        }

        // 加载脚本类
        val classLoader = URLClassLoader(arrayOf(tmpJarFile.toURI().toURL()))
        val runnableClass: Class<*> = classLoader.loadClass(runnableClassName)
        loadedJarPatchClassSet.add(runnableClass.name)

        // 执行脚本
        logger.info("Executing jar $tmpJarFile, class=$runnableClass")
        (runnableClass.getDeclaredConstructor().newInstance() as Runnable).run()

        // 返回执行结果
        sender.tell(SCRIPT_DAEMON_OK_RESPONSE, ActorRef.noSender())
    }

}
