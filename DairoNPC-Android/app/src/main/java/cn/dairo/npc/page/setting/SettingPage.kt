package cn.dairo.npc.page.setting

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.core.graphics.drawable.toBitmap
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavController
import cn.dairo.npc.extension.relaunch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NpcConfigPage(
    navController: NavController,
    modifier: Modifier = Modifier
) {
    Scaffold(
        topBar = {
            TopAppBar(
                modifier = modifier.background(Color.Red),
                title = {
                    Text("配置DairoNPC", color = MaterialTheme.colorScheme.onPrimary)
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primary
                )
            )
        }
    ) { innerPadding ->
        ContentView(navController,Modifier.padding(innerPadding))
    }
}

@Composable
private fun ContentView(
    navController: NavController,
    modifier: Modifier = Modifier,
    vm: SettingViewModel = viewModel()
) {
    val state by vm.state.collectAsState()
    Column(modifier = modifier.padding(10.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        val context = LocalContext.current
        val drawable = context.packageManager
            .getApplicationIcon(context.packageName)

        Spacer(Modifier.height(10.dp))
        Image(
            bitmap = drawable.toBitmap().asImageBitmap(),
            modifier = Modifier.size(100.dp),
            contentDescription = "logo"
        )
        Spacer(Modifier.height(20.dp))
        OutlinedTextField(
            value = state.npcSetting.host,
            onValueChange = { host ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(host = host)
                    )
                }
            },
            label = {
                Text("服务器")
            },
            placeholder = {
                Text("必填;例:192.168.0.100;www.xxxxxx.com")
            },
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(Modifier.height(10.dp))

        OutlinedTextField(
            value = state.npcSetting.tcpPort,
            onValueChange = { tcpPort ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(tcpPort = tcpPort)
                    )
                }
            },
            label = {
                Text("TCP端口")
            },
            placeholder = {
                Text("默认1881")
            },
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(Modifier.height(10.dp))

        OutlinedTextField(
            value = state.npcSetting.udpPort,
            onValueChange = { udpPort ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(udpPort = udpPort)
                    )
                }
            },
            label = {
                Text("UDP端口")
            },
            placeholder = {
                Text("默认1882")
            },
            modifier = Modifier.fillMaxWidth()
        )

        Spacer(Modifier.height(20.dp))

        OutlinedTextField(
            value = state.npcSetting.key,
            onValueChange = { key ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(key = key)
                    )
                }
            },
            label = {
                Text("秘钥")
            },
            placeholder = {
                Text("必填")
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
            Text("保存并连接")
        }
    }
}