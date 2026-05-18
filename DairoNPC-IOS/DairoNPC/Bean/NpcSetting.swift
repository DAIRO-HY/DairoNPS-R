struct NpcSetting  : Codable{
    
    /**
     * 服务器
     */
    var host: String = ""
    
    /**
     * tcp端口
     */
    var tcpPort: String = ""
    
    /**
     * udp端口
     */
    var udpPort: String = ""
    
    /**
     * 秘钥
     */
    var key: String = ""
    
//    static var new12: NpcSetting{
//        get{
//            NpcSetting(host:"",tcpPort:"",udpPort:"",key:"")
//        }
//    }
}
