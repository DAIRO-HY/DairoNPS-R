package cn.dairo.npc.bean

/**
 * Npc状态信息
 */
data class NpcStatus(

    /**
     * NPC打开状态
     */
    val isOpened: Boolean = false,

    /**
     * NPC正在运行
     */
    val isRunning: Boolean = false,

    /**
     * NPC连接消息
     */
    val connectMsg: String = "",

    /**
     * 当前桥接数量
     */
    val bridgeCount: Short = 0,

    /**
     * 当前连接池数量
     */
    val poolCount: Short = 0,

    /**
     * 入网流量
     */
    val inLen: Long = 0,

    /**
     * 出网流量
     */
    val outLen: Long = 0,
)