package cn.dairo.npc

/**
 * Npc状态信息
 */
data class NpcStatusInfo(

    /**
     * 服务器端id
     */
    val npsId: Long,

    /**
     * 当前桥接数量
     */
    val bridgeCount: Int,

    /**
     * 当前连接池数量
     */
    val poolCount: Int
)