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
        VStack{
            Spacer()
            Image("logo")
                .resizable()
                .frame(width: 80, height: 80)
                .cornerRadius(14)   // 设置圆角半径
                .padding(32)
            TextBox(hide: "服务器",text: $vm.state.npcSetting.host,icon:"server.rack")
            TextBox(hide: "TCP端口", text: $vm.state.npcSetting.tcpPort, icon:"guidepoint.vertical.numbers")
            TextBox(hide: "UDP端口", text: $vm.state.npcSetting.udpPort, icon:"guidepoint.vertical.numbers")
            TextBox(hide: "秘钥", text: $vm.state.npcSetting.key, icon:"key.shield")
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
            .background(Color("btn.bg"))
            .cornerRadius(6)
            .overlay(// 设置边框样式
                RoundedRectangle(cornerRadius: 6)
                    .stroke(.black, lineWidth: 1)
            )
            Spacer()
        }
        .padding(.all)
        .safeAreaInset(edge: .top){
            HStack {
                Text("配置DairoNPC").font(.headline).frame(maxWidth: .infinity)
            }
            .padding()
            .background(.ultraThinMaterial)
        }
    }
    
    private func TextBox(hide:String = "",text:Binding<String>,icon:String = "")->some View {
        HStack {
            Image(systemName: icon)
                .frame(width:34)
            //            .foregroundColor(Color.gl.textContent)
                .font(.system(.body))
            
            TextField(hide, text: text)
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
                 //            .foregroundColor(Color.gl.borderPrimary)
        )
    }
}

#Preview {
    SettingPage()
}
