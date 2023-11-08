package xyz.ariane.util.dbdirty

import com.esotericsoftware.kryo.Kryo

interface KryoCheckReg {

    fun register(kryo: Kryo)

}