package cn.dairo.npc

object RustBridge {

    init {
        System.loadLibrary("npc_android")
    }

    /**
     * 启动NPC服务
     */
    external fun start(
        host: String,
        tcpPort: Short,
        udpPort: Short,
        key: String
    )

    /**
     * 停止
     */
    external fun stop()

    external fun getHello(input: String): String

    external fun getStatusInfo(): NpcStatusInfo

}