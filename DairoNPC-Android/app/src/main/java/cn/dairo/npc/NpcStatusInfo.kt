package cn.dairo.npc

/**
 * Npc状态信息
 */
data class NpcStatusInfo(

    /**
     * NPC打开状态
     */
    val isOpened: Boolean,

    /**
     * NPC正在运行
     */
    val isRunning: Boolean,

    /**
     * NPC连接消息
     */
    val connectMsg: String,

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
    val poolCount: Int,
)