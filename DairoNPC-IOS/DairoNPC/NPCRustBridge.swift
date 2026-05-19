class NPCRustBridge {
    
    /**
     * 启动NPC服务
     */
    static func open(
        _ host: String,
        _ tcpPort: Int16,
        _ udpPort: Int16,
        _ key: String
    ){
        host.withCString{host in
            key.withCString{key in
                npc_open(host, 1881, 1881, key)
            }
        }
    }
    
    /**
     * 停止
     */
    static func close(){
        npc_close()
    }
    
    /**
     * 获取状态信息
     */
    static func getStatus() -> NpcStatus{
        let ptr = npc_get_status()!
        let rustStatus = ptr.pointee
        let status = NpcStatus(
            isOpened: rustStatus.is_opened,
            isRunning: rustStatus.is_running,
            connectMsg: String(cString: rustStatus.connect_msg),
            bridgeCount: rustStatus.bridge_count,
            poolCount: rustStatus.pool_count,
            inLen: Int64(rustStatus.in_len),
            outLen: Int64(rustStatus.out_len),
            clientId: Int64(rustStatus.client_id)
        )
        npc_free_status(ptr)
        return status
        
    }
    
    /**
     * 获取Npc信息
     */
    static func getInfo() -> NpcInfo{
        let ptr = npc_get_info()!
        let rustInfo = ptr.pointee
        let info = NpcInfo(
            version: String(cString: rustInfo.version)
        )
        npc_free_info(ptr)
        return info
    }
    
}
