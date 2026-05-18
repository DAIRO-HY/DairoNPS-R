
/**
 * NPC信息
 */
struct NpcInfo{
    
    /**
     * 客户端id
     */
    let clientId: Int64
    
    /**
     * NPC版本
     */
    let version: String
    
    init(){
        self.clientId = 0
        self.version = ""
    }
    
    init(clientId: Int64, version: String) {
        self.clientId = clientId
        self.version = version
    }
}
