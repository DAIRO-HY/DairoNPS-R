package cn.dairo.npc

import cn.dairo.npc.bean.NpcInfo
import cn.dairo.npc.bean.NpcStatus

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

    /**
     * 获取状态信息
     */
    external fun getStatus(): NpcStatus

    /**
     * 获取Npc信息
     */
    external fun getInfo(): NpcInfo

}