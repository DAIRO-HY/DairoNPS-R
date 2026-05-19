
import Combine
import Foundation
import SwiftUI
class HomeViewModel: ObservableObject {
    
    /**
     * 记录上次获取到的入网流量,用来统计网速
     */
    private var lastInLen: Int64 = 0
    
    /**
     * 记录上次获取到的出网流量,用来统计网速
     */
    private var lastOutLen: Int64 = 0
    
    private var loopGetStatusJob: Task<Void, Never>?
    
    @Published var state: HomeState
    
    init() {
        self.state = HomeState(
            npcSetting: NpcRepository.loadSetting(),////配置信息
            npcInfo: NPCRustBridge.getInfo(),//客户端信息,
            npcStatus: NpcStatus(),
            statusIcon: "wifi",
            statusLabel: "",
            isOpened: NpcRepository.isOpened(),
            statusColor: Color.green,
            inSpeed: "",
            outSpeed: ""
        )
        if NpcRepository.isOpened() {
            self.openNpc()
        }
    }
    
    /**
     * 轮询获取NPC状态
     */
    func loopGetStatus() {
        self.loopGetStatusJob?.cancel()
        self.loopGetStatusJob = Task{
            while (true) {
                self.updateStatus()
                do{
                    try await Task.sleep(nanoseconds: 1_000_000_000)
                } catch {
                    break
                }
            }
        }
    }
    
    /**
     * 停止循环获取状态
     */
    func cancelLoopGetStatus() {
        self.loopGetStatusJob?.cancel()
        self.lastInLen = 0
        self.lastOutLen = 0
    }
    
    private func updateStatus() {
        let npcStatus = NPCRustBridge.getStatus()
        let inSpeed = if lastInLen == 0 {
            Int64(0)
        } else {
            npcStatus.inLen - lastInLen
        }
        let outSpeed = if lastOutLen == 0 {
            Int64(0)
        } else {
            npcStatus.outLen - lastOutLen
        }
        self.lastInLen = npcStatus.inLen
        self.lastOutLen = npcStatus.outLen
        self.state.npcStatus = npcStatus
        self.state.statusLabel = npcStatus.isRunning ? "⚫︎连接正常" : "⚫︎连接断开"
        self.state.statusColor = npcStatus.isRunning ? Color("status_label_success") : Color("status_label_fail")
        self.state.statusIcon = npcStatus.isRunning ? "wifi" : "wifi.slash"
        self.state.inSpeed = inSpeed.readableSize + "/s"
        self.state.outSpeed = outSpeed.readableSize + "/s"
    }
    
    /**
     * 标记为重新配置
     */
    func onResetClick() {
        NPCRustBridge.close()
        NpcRepository.saveSet(false)
        DairoNPCApp.relaunch()
    }
    
    /**
     * 打开/关闭NPC服务
     */
    func onOpenNpcClick() {
        if (NpcRepository.isOpened()) {//关闭NPC
            NpcRepository.setOpened(false)
            NPCRustBridge.close()
        } else {//打开NPC
            NpcRepository.setOpened(true)
            openNpc()
        }
        self.state.isOpened = NpcRepository.isOpened()
    }
    
    private func openNpc() {
        let npcSetting = self.state.npcSetting
        NPCRustBridge.open(
            npcSetting.host,
            Int16(npcSetting.tcpPort)!,
            Int16(npcSetting.udpPort)!,
            npcSetting.key
        )
    }
}
