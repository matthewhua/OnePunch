package xyz.ariane.util.memodbupgrade

import java.util.HashMap

data class CheckDeserializeData(
    val aav: Int,
    val members: HashMap<String, CheckData>
)