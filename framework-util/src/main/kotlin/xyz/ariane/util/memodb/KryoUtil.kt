package xyz.ariane.util.memodb

import com.esotericsoftware.kryo.Kryo
import com.esotericsoftware.kryo.io.Input
import com.esotericsoftware.kryo.io.Output
import com.esotericsoftware.kryo.util.Pool
import xyz.ariane.util.annotation.AllOpen
import java.io.ByteArrayInputStream

data class KryoBytes(
    val bytes: ByteArray,
    val dataLength: Int
)

@AllOpen
open class KryoUtil(val externalInit: (kryo: Kryo) -> Unit) {

    private val kryos: ThreadLocal<Kryo> = object : ThreadLocal<Kryo>() {

        override fun initialValue(): Kryo {
            // Configure the Kryo instance.
            val kryo = Kryo()
            externalInit(kryo)
            return kryo
        }
    }

    var outputPool: Pool<Output> = object : Pool<Output>(true, false, 32) {
        override fun create(): Output {
            return Output(1024, -1)
        }
    }

    var inputPool: Pool<Input> = object : Pool<Input>(true, false, 32) {
        override fun create(): Input {
            return Input(-1)
        }
    }

    val registerMap = mutableMapOf<Class<*>, Int>()
    var registerNo = 1

    /**
     * 注册序列化类
     */
    fun register(kryo: Kryo, clazz: Class<*>, id: Int = -1) {
        if (registerMap[clazz] != null) {
            throw RuntimeException("kryo类注册表中，重复注册了 ${clazz.name}")
        }

        var classId = id
        if (id == -1) {
            classId = registerNo
            registerNo++
        }
        registerMap[clazz] = classId

        kryo.register(clazz, classId)
    }

    /**
     * 序列化
     */
    open fun serialize(from: Any): KryoBytes {
        val out = outputPool.obtain()

        kryos.get().writeObject(out, from)

        val bytes = out.toBytes()
        val kb = KryoBytes(bytes, out.position())

        outputPool.free(out)

        return kb
    }

    fun <T> deserialize(from: ByteArray, clazz: Class<T>): T {
        val bis = ByteArrayInputStream(from)
        val input = Input(bis)

        val obj = kryos.get().readObject(input, clazz)

        input.close()
        bis.close()

        return obj
    }
}