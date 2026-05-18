import Combine
import Foundation
class SettingViewModel: ObservableObject {
    @Published var state = SettingState()
    func save(){
        var npcSetting = self.state.npcSetting
        if(npcSetting.tcpPort.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty){
            npcSetting.tcpPort = "1881"
        }
        if(npcSetting.udpPort.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty){
            npcSetting.udpPort = "1881"
        }
        if(npcSetting.host.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty){
            self.state.error = "服务器地址不能为空"
            return
        }
        if(npcSetting.key.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty){
            self.state.error = "秘钥不能为空"
            return
        }
        if Int16(npcSetting.tcpPort).map({ $0 > 0 }) != true {
            self.state.error = "TCP端口必须为正整数且小于65536"
            return
        }
        if Int16(npcSetting.udpPort).map({ $0 > 0 }) != true {
            self.state.error = "UDP端口必须为正整数且小于65536"
            return
        }
        NpcRepository.saveSetting(npcSetting)
        NpcRepository.saveSet(true)
        NpcRepository.setOpened(true)
        DairoNPCApp.relaunch()
    }
}
