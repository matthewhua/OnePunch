package xyz.ariane.util.i18n

import xyz.ariane.util.enumeration.EnumUtils
import xyz.ariane.util.enumeration.NamedEnum

enum class Language(override val enumName: String) : NamedEnum {

    zh_CN("简体中文"),

    zh_TW("台湾繁体"),

    zh_AP("亚太繁体"),

    vi_VN("越南越南语"),

    ko_KR("韩国韩语"),

    ja_JP("日本日语"),

    tr_TR("泰国泰语");

    companion object {

        fun getByZhName(name: String): Language {
            return EnumUtils.getByName(name, values())
        }
    }
}
