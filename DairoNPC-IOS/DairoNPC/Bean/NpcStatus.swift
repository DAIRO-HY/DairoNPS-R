
// NPC实时状态信息
struct NpcStatus {
    
    /**
     * NPC打开状态
     */
    let isOpened: Bool

    /**
     * NPC正在运行
     */
    let isRunning: Bool

    /**
     * NPC连接消息
     */
    let connectMsg: String

    /**
     * 当前桥接数量
     */
    let bridgeCount: UInt16

    /**
     * 当前连接池数量
     */
    let poolCount: UInt16

    /**
     * 入网流量
     */
    let inLen: Int64

    /**
     * 出网流量
     */
    let outLen: Int64
    
    /**
     * 客户端id
     */
    let clientId: Int64
    init() {
        self.isOpened = false
        self.isRunning = false
        self.connectMsg = ""
        self.bridgeCount = 0
        self.poolCount = 0
        self.inLen = 0
        self.outLen = 0
        self.clientId = 0
    }
    init(isOpened: Bool, isRunning: Bool, connectMsg: String, bridgeCount: UInt16, poolCount: UInt16, inLen: Int64, outLen: Int64, clientId: Int64) {
        self.isOpened = isOpened
        self.isRunning = isRunning
        self.connectMsg = connectMsg
        self.bridgeCount = bridgeCount
        self.poolCount = poolCount
        self.inLen = inLen
        self.outLen = outLen
        self.clientId = clientId
    }
}
