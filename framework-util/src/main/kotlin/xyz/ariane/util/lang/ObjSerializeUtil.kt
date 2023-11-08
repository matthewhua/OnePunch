package xyz.ariane.util.lang

import xyz.ariane.util.annotation.AllOpen
import java.io.*

var objSerializeUtil = ObjSerializeUtil()

@AllOpen
class ObjSerializeUtil {
    fun obj2Bytes(obj: Serializable): ByteArray {
        val bo = ByteArrayOutputStream()
        val oo = ObjectOutputStream(bo)
        oo.writeObject(obj)
        val bytes = bo.toByteArray()
        bo.close()
        oo.close()
        return bytes
    }

    fun bytes2Obj(bytes: ByteArray): Any? {
        try {
            val bi = ByteArrayInputStream(bytes)
            val oi = ObjectInputStream(bi)
            val obj = oi.readObject()
            bi.close()
            oi.close()
            return obj
        } catch (ex: Exception) {
            ex.printStackTrace()
            return null
        }
    }
}