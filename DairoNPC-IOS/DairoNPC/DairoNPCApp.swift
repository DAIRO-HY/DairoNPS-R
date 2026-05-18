import SwiftUI

@main
struct DairoNPCApp: App {
    
    //页面切换通知标记
    private static let REFRESH_PAGE = "REFRESH_PAGE"
    
    //当前显示的页面
    @State private var refreshID = 0
    var body: some Scene {
        WindowGroup {
            ZStack{
                if NpcRepository.isSet(){
                    HomePage()
                }else{
                    SettingPage()
                }
            }
            .id(self.refreshID)
            .onReceive(NotificationCenter.default.publisher(for: Notification.Name(DairoNPCApp.REFRESH_PAGE))){ name in
                self.refreshID += 1
            }
        }
    }
    
    //页面重新加载
    static func relaunch(){
        NotificationCenter.default.post(name: Notification.Name(DairoNPCApp.REFRESH_PAGE), object: nil)
    }
}
