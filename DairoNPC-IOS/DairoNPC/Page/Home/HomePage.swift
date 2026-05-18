import SwiftUI

struct HomePage: View {
    @StateObject private var vm = HomeViewModel()
    var body: some View {
        GeometryReader { geo in
            VStack{
                npcStatusView(geo.size.width)
                Spacer().frame(height: 20)
                dataSizeView()
                Spacer().frame(height: 20)
                connectCountView()
                Spacer().frame(height: 20)
                systemInfoView()
            }
            .padding(.all)
            .safeAreaInset(edge: .top){
                HStack {
                    Spacer().frame(maxWidth: .infinity)
                    Text("DairoNPC").font(.headline).frame(maxWidth: .infinity)
                    Button(action: self.vm.onResetClick){
                        Image(systemName: "gearshape")
                    }
                    .frame(maxWidth: .infinity, alignment: .trailing)
                    //                    .background(Color.red)
                }
                .padding()
                .background(.ultraThinMaterial)
            }
            .onAppear{
                self.vm.loopGetStatus()
            }
        }
    }
    
    func cardView<Content: View>(
        @ViewBuilder content: () -> Content
    ) -> some View {
        HStack {
            content()
        }
        //        .background(Color(.systemBackground))
        .background(Color.secondary.opacity(0.2))
        .cornerRadius(8)
    }
    
    /**
     * NPC信息标签
     */
    private func npcInfoView(_ title: String, _ value: String) -> some View {
        HStack{
            Text(title).foregroundColor(.secondary)
            Text(value).frame(maxWidth: .infinity,alignment: .trailing)
        }
    }
    
    private func npcStatusView(_ screenWidth: Double) -> some View{
        self.cardView {
            HStack{
                let enableWidth = screenWidth - 20
                VStack{
                    Image(systemName: self.vm.state.statusIcon)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 60, height: 60)
                        .foregroundColor(self.vm.state.statusColor)
                    Spacer().frame(height: 30)
                    Button(action:self.vm.onOpenNpcClick){
                        Text(self.vm.state.isOpened ? "断开连接" : "启动连接")
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
                }.frame(width: enableWidth / 3)
                    .padding(.all, 10)
                VStack{
                    Text(self.vm.state.statusLabel).frame(maxWidth: .infinity, alignment: .leading)
                        .font(.system(size: 30))
                        .foregroundColor(self.vm.state.statusColor)
                    Text(self.vm.state.npcStatus.connectMsg).frame(maxWidth: .infinity, alignment: .leading)
                    self.npcInfoView("服务器", self.vm.state.npcSetting.host)
                    self.npcInfoView("TCP端口", self.vm.state.npcSetting.tcpPort)
                    self.npcInfoView("UDP端口", self.vm.state.npcSetting.udpPort)
                    self.npcInfoView("连接秘钥", self.vm.state.npcSetting.key)
                }
                .padding(.all, 10)
            }
        }
    }
    
    private func dataSizeView() -> some View{
        self.cardView {
            VStack{
                Text("流量统计").frame(maxWidth: .infinity,alignment: .leading).foregroundColor(.secondary)
                HStack{
                    Image(systemName: "arrow.up").resizable().frame(width: 15,height: 20)
                    VStack{
                        Text("上行流量").foregroundColor(.secondary)
                        Text(self.vm.state.npcStatus.outLen.readableSize)
                        Text(self.vm.state.outSpeed).foregroundColor(.secondary)
                    }.frame(maxWidth: .infinity)
                    Divider().frame(height: 30)
                    VStack{
                        Text("下行流量").foregroundColor(.secondary)
                        Text(self.vm.state.npcStatus.inLen.readableSize)
                        Text(self.vm.state.inSpeed).foregroundColor(.secondary)
                    }.frame(maxWidth: .infinity)
                    Image(systemName: "arrow.down").resizable().frame(width: 15,height: 20)
                }
            }
            .padding(.all,10)
        }
    }
    
    private func connectCountView() -> some View{
        
        self.cardView {
            VStack{
                Text("连接数量").frame(maxWidth: .infinity,alignment: .leading).foregroundColor(.secondary)
                HStack{
                    Image(systemName: "arrow.up.arrow.down").resizable().frame(width: 20,height: 20)
                    VStack{
                        Text("桥接数").foregroundColor(.secondary)
                        Text(String(self.vm.state.npcStatus.bridgeCount))
                    }.frame(maxWidth: .infinity)
                    Divider().frame(height: 30)
                    VStack{
                        Text("连接池").foregroundColor(.secondary)
                        Text(String(self.vm.state.npcStatus.poolCount))
                    }.frame(maxWidth: .infinity)
                    Image(systemName: "square.stack.3d.up").resizable().frame(width: 20,height: 20)
                }
            }
            .padding(.all,10)
        }
    }
    
    private func systemInfoView() -> some View{
        self.cardView {
            VStack{
                Text("系统信息").frame(maxWidth: .infinity,alignment: .leading).foregroundColor(.secondary)
                HStack{
                    VStack{
                        Text("NPC版本").foregroundColor(.secondary)
                        Text("1.0.0")
                    }.frame(maxWidth: .infinity)
                    Divider().frame(height: 30)
                    VStack{
                        Text("客户端ID").foregroundColor(.secondary)
                        Text("1")
                    }.frame(maxWidth: .infinity)
                }
            }
            .padding(.all,10)
        }
    }
}

#Preview {
    HomePage()
}
