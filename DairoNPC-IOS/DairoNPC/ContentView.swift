//
//  ContentView.swift
//  DairoNPC
//
//  Created by zhoulq on 2026/05/07.
//

import SwiftUI

struct ContentView: View {
    
    @State var count: Int32 = 0
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Hello, world!\(count)")
            Button("START"){
                NPCRustBridge.open("192.168.3.63", 1881, 1881, "dvdvdf")
            }
            Button("STOP"){
                NPCRustBridge.close()
            }
            Button("STRUCT_TEST"){
                for _ in 0 ... 1{
                    print(NPCRustBridge.getStatus())
                }
            }
        }
        .padding()
        .onAppear{
            //            Task{
            //                while true{
            //                    try? await Task.sleep(nanoseconds: 1_000_000_000)
            //                    count = npc_bridge_count()
            //                }
            //            }
        }
    }
}

#Preview {
    ContentView()
}
