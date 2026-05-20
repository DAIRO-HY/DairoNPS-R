//
//  SettingPage.swift
//  DairoNPC
//
//  Created by zhoulq on 2026/05/15.
//

import SwiftUI

struct SettingPage: View {
    
    @StateObject var vm = SettingViewModel()
    
    var body: some View {
        ScrollView {
            VStack{
                Image("logo")
                    .resizable()
                    .frame(width: 60, height: 60)
                    .cornerRadius(12)   // 设置圆角半径
                    .padding(32)
                TextEditBox($vm.state.npcSetting.host, hide: "服务器", icon: "server.rack")
                TextEditBox($vm.state.npcSetting.tcpPort, hide: "TCP端口(默认1881)", icon: "number.square")
                TextEditBox($vm.state.npcSetting.udpPort, hide: "UDP端口(默认1882)", icon: "number.square")
                TextEditBox($vm.state.npcSetting.key, hide: "连接秘钥", icon: "key.viewfinder")
                if self.vm.state.error != nil{
                    Spacer().frame(height: 10)
                    Text(self.vm.state.error!).foregroundColor(Color.red).frame(maxWidth: .infinity,alignment: .leading)
                }
                Spacer().frame(height: 30)
                Button(action:self.vm.save){
                    Text("保存并连接")
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 10)
                        .font(.body)
                        .foregroundColor(Color.white)
                }
                .background(Color("bg_primary"))
                .cornerRadius(6)
                .overlay(// 设置边框样式
                    RoundedRectangle(cornerRadius: 6)
                        .stroke(.black, lineWidth: 1)
                )
            }
            .frame(maxWidth: 360)
            .padding()
            .safeAreaInset(edge: .top){
                HStack {
                    Text("配置DairoNPC").font(.headline).frame(maxWidth: .infinity).foregroundColor(.white)
                }
                .padding()
                .background(Color("bg_primary"))
            }
        }
    }
    
    /**
     * 文本输入框
     */
    private func TextEditBox(_ value: Binding<String>, hide: String = "", icon: String = "")->some View {
        HStack {
            Image(systemName: icon)
                .frame(width:34)
                .foregroundColor(Color.secondary)
                .font(.system(.body))
            
            TextField(hide, text: value)
                .keyboardType(.emailAddress) // 确保输入法切换
                .autocapitalization(.none) // 可选：关闭自动大写
                .disableAutocorrection(true) // 可选：关闭自动校正
                .font(.body)
            //                    .foregroundColor(Color.gl.textContent)
        }
        .padding(.vertical,10)
        .padding(.trailing)
        //                .background(Color.white)
        .cornerRadius(6)
        .overlay(RoundedRectangle(cornerRadius: 6).stroke(lineWidth: 1)
            .foregroundColor(Color("bg_primary"))
        )
    }
}

#Preview {
    SettingPage()
}
