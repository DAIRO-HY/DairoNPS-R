package cn.dairo.npc.bean

data class NpcSetting (

    /**
     * 服务器
     */
    val host: String = "",

    /**
     * tcp端口
     */
    val tcpPort: String = "",

    /**
     * udp端口
     */
    val udpPort: String = "",

    /**
     * 秘钥
     */
    var key: String = "",
)