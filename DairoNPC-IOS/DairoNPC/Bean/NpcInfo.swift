
/**
 * NPC信息
 */
struct NpcInfo{
    
    /**
     * NPC版本
     */
    let version: String
    
    init(){
        self.version = ""
    }
    
    init(version: String) {
        self.version = version
    }
}
