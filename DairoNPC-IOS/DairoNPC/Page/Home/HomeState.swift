import SwiftUI
struct HomeState{
    
//    //NPC客户端配置信息
//    let npcSetting = NpcRepository.loadSetting()
//    
//    //NPC客户端信息
//    let npcInfo = NpcInfo()
//    
//    //NPC连接信息
//    var npcStatus = NpcStatus()
//    
//    //状态标签文字
//    var statusIcon = "wifi"
//    
//    //状态标签文字
//    var statusLabel = ""
//    
//    //标记是否已经打开过
//    let isOpened = NpcRepository.isOpened()
//    
//    //状态颜色
//    var statusColor = Color.green
//    
//    //入网网速
//    var inSpeed = ""
//    
//    //出网网速
//    var outSpeed = ""
    
    //NPC客户端配置信息
    let npcSetting: NpcSetting
    
    //NPC客户端信息
    let npcInfo: NpcInfo
    
    //NPC连接信息
    var npcStatus: NpcStatus
    
    //状态标签文字
    var statusIcon: String
    
    //状态标签文字
    var statusLabel: String
    
    //标记是否已经打开过
    var isOpened: Bool
    
    //状态颜色
    var statusColor: Color
    
    //入网网速
    var inSpeed: String
    
    //出网网速
    var outSpeed: String
}
