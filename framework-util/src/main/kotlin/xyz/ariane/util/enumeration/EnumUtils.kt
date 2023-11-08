package xyz.ariane.util.enumeration

import com.google.common.base.Preconditions
import xyz.ariane.util.lang.notNullOrBlank

object EnumUtils {

    /**
     * @param name   名字
     * @param values 全部枚举值
     * @return 名字对应的枚举值
     * @throws IllegalArgumentException 如果找不到名字对应的枚举值
     */
    fun <T : NamedEnum> getByName(name: String, values: Array<T>): T {
        Preconditions.checkArgument(notNullOrBlank(name), "名字不能为空")
        Preconditions.checkNotNull(values)
        Preconditions.checkArgument(values.size > 0)

        for (e in values) {
            if (e.enumName == name.trim { it <= ' ' }) {
                return e
            }
        }
        throw IllegalArgumentException(String.format("类型 %s 中找不到名字为 %s 的元素", values[0].javaClass.name, name))
    }

    /**
     * @param str    名字
     * @param values 全部枚举值
     * @return 字符串是否是否可以转换为枚举元素
     */
//    fun <T : Enum<*>> isInEnum(str: String, values: Array<T>?): Boolean {
//        return values != null && Arrays.stream(values).anyMatch { value -> value.name == str }
//    }

}
