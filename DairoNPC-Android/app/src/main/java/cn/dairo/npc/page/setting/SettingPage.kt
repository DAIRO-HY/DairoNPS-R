package cn.dairo.npc.page.setting

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavController
import cn.dairo.npc.extension.relaunch

@Composable
fun NpcConfigPage(
    navController: NavController,
    modifier: Modifier = Modifier,
    vm: SettingViewModel = viewModel()
) {
    val context = LocalContext.current
    val state by vm.state.collectAsState()
    Column(modifier = modifier.padding(10.dp)) {
        OutlinedTextField(
            value = state.npc.host,
            onValueChange = { host ->
                vm.update {
                    state.copy(
                        npc = state.npc.copy(host = host)
                    )
                }
            },

            label = {
                Text("服务器")
            },
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(Modifier.height(10.dp))

        OutlinedTextField(
            value = state.npc.tcpPort,
            onValueChange = { tcpPort ->
                vm.update {
                    state.copy(
                        npc = state.npc.copy(tcpPort = tcpPort)
                    )
                }
            },
            label = {
                Text("TCP端口")
            },
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(Modifier.height(10.dp))

        OutlinedTextField(
            value = state.npc.udpPort,
            onValueChange = { udpPort ->
                vm.update {
                    state.copy(
                        npc = state.npc.copy(udpPort = udpPort)
                    )
                }
            },
            label = {
                Text("UDP端口")
            },
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(Modifier.height(20.dp))

        OutlinedTextField(
            value = state.npc.key,
            onValueChange = { key ->
                vm.update {
                    state.copy(
                        npc = state.npc.copy(key = key)
                    )
                }
            },
            label = {
                Text("秘钥")
            },
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(Modifier.height(20.dp))

        Button(
            onClick = {
                vm.save {
                    navController.relaunch("home")
                }
            },
            modifier = Modifier.fillMaxWidth()
        ) {
            Text("保存")
        }
    }
}