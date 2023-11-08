package xyz.ariane.util.net

import com.google.common.collect.Lists
import java.net.Inet4Address
import java.net.InetAddress
import java.net.NetworkInterface
import java.util.*
import java.util.regex.Pattern

data class LocalIpInfo(
    val displayName: String,
    val ips: List<Pair<InetAddress, String>>
)

class NetUtils {

    companion object {
        /**
         * 找到所有的内网IP
         */
        fun findAllLocalIp(): List<String> {
            // 获得本机ip
            val ipList = LinkedList<String>()
            val inetAddrList = allLocalInetAddrs()
            inetAddrList.forEach {
                if (!it.isLoopbackAddress && !it.isLinkLocalAddress) {
                    ipList.add(it.hostAddress)
                }
            }

            return ipList
        }

        /**
         * 获取本地IP信息
         * @return List<LocalIpInfo>
         */
        fun findLocalIpInfos(): List<LocalIpInfo> {
            // 获得本机ip
            val ipList = mutableListOf<LocalIpInfo>()
            for (nif in NetworkInterface.getNetworkInterfaces()) {
                if (nif.isLoopback || !nif.isUp) {
                    continue
                }

                val ips = mutableListOf<Pair<InetAddress, String>>()
                for (addr in nif.inetAddresses) {
                    if (addr.isLinkLocalAddress) {
                        continue
                    }

                    ips.add(Pair(addr, addr.hostAddress))
                }

                val info = LocalIpInfo(nif.displayName, ips)
                ipList.add(info)
            }

            return ipList
        }

        //ip转long
        fun ipToLong(ip: String): Long? {
            val addrList = ip.split('.')
            if (addrList.size != 4) {
                return null
            }

            val fetchAddr = { addrStr: String ->
                val addrInt = addrStr.toLongOrNull()
                if (addrInt != null && addrInt in 0..(2 shl 8 - 1)) {
                    addrInt
                } else {
                    null
                }
            }

            val addr1 = fetchAddr(addrList[0]) ?: return null
            val addr2 = fetchAddr(addrList[1]) ?: return null
            val addr3 = fetchAddr(addrList[2]) ?: return null
            val addr4 = fetchAddr(addrList[3]) ?: return null

            return (addr1 shl 24) + (addr2 shl 16) + (addr3 shl 8) + addr4
        }

        //掩码位转long类型ip
        fun maskCapacityToLong(maskCapacity: Int): Long {
            var mask = 0L
            for (i in 0 until 4) {
                var str = ""
                val index = i * 8
                for (j in 1..8) {
                    str += if (j + index <= maskCapacity) "1" else "0"
                }
                mask += Integer.parseInt(str, 2).toLong() shl (24 - index)
            }
            return mask
        }

        // 检测IP格式 例如：255.255.255.255/24
        fun checkIpWithMask(ip: String): Boolean {
            if (ip.length < 7 || ip.length > 18 || "" == ip.trim()) {
                return false
            }

            val regex =
                "^([1-9]|[1-9]\\d|1\\d{2}|2[0-4]\\d|25[0-5])(\\.(\\d|[1-9]\\d|1\\d{2}|2[0-4]\\d|25[0-5])){3}(/([1-9]|[1-2]\\d|3[0-2]))?$"
            val pattern = Pattern.compile(regex)
            val matcher = pattern.matcher(ip)

            return matcher.find()
        }

        // 检测IP格式 例如：255.255.255.*
        fun checkIpWithWildcard(ip: String): Boolean {
            if (ip.length < 7 || ip.length > 15 || "" == ip.trim()) {
                return false
            }

            val regex =
                "^([1-9]|[1-9]\\d|1\\d{2}|2[0-4]\\d|25[0-5]|\\*)(\\.(\\d|[1-9]\\d|1\\d{2}|2[0-4]\\d|25[0-5]|\\*)){3}$"
            val pattern = Pattern.compile(regex)
            val matcher = pattern.matcher(ip)

            return matcher.find()
        }

        // 检查分布式IP和端口 格式 ip:port,ip:port,ip:port...
        fun checkMultipleIpWithPort(ips: String): Boolean {
            val ipList = ips.split(",")
            for (ip in ipList) {
                val checkRs = checkIpWithPort(ip)
                if (!checkRs) {
                    return false
                }
            }

            return true
        }

        // 检测IP和端口
        fun checkIpWithPort(ip: String): Boolean {
            val splitIp = ip.split(":")
            if (splitIp.size != 2) {
                return false
            }

            val port = splitIp[1].toIntOrNull()
            if (port == null || port < 1025 || port > 65534) {
                return false
            }

            return checkIp(splitIp[0])
        }

        // 检测IP格式
        fun checkIp(ip: String): Boolean {
            if (ip.length < 7 || ip.length > 15 || "" == ip.trim()) {
                return false
            }

            val regex = "^([1-9]|[1-9]\\d|1\\d{2}|2[0-4]\\d|25[0-5])(\\.(\\d|[1-9]\\d|1\\d{2}|2[0-4]\\d|25[0-5])){3}$"
            val pattern = Pattern.compile(regex)
            val matcher = pattern.matcher(ip)

            return matcher.find()
        }

        //检测ip是否在同一子网下
        fun checkIpInSameMask(ip: String, maskCapacity: Int, checkIp: String): Boolean {
            if (maskCapacity !in 1..32) {
                return false
            }
            val mask = maskCapacityToLong(maskCapacity)

            val ipBinary = ipToLong(ip) ?: return false
            val checkIpBinary = ipToLong(checkIp) ?: return false

            return (ipBinary and mask) == (checkIpBinary and mask)
        }

    }

}

/**
 * 获取内网非loopback的IPv4地址
 */
fun getSiteLocalIp(): String = allLocalInetAddrs()
    .filter { it.isSiteLocalAddress && !it.isLoopbackAddress && it is Inet4Address }
    .map(InetAddress::getHostAddress)
    .first()

/**
 * 获取本机外网IP
 */
fun getNonSiteLocalIp(): String = allLocalInetAddrs()
    .filter { !it.isSiteLocalAddress && it is Inet4Address }
    .map(InetAddress::getHostAddress)
    .first()

fun allLocalInetAddrs(): List<InetAddress> {
    val inetAddrList = Lists.newArrayListWithExpectedSize<InetAddress>(2)
    for (nif in NetworkInterface.getNetworkInterfaces()) {
        for (addr in nif.inetAddresses) {
            inetAddrList.add(addr)
        }
    }
    return inetAddrList
}

fun main(args: Array<String>) {
    getSiteLocalIp().let(::println)
}