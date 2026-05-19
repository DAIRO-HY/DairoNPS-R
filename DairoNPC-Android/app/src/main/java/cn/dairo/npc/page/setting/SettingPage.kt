package cn.dairo.npc.page.setting

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Key
import androidx.compose.material.icons.filled.Pin
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavController
import cn.dairo.npc.R
import cn.dairo.npc.extension.extraColors
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
                    Text("配置DairoNPC", color = Color.White)
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.extraColors.bgPrimary
                )
            )
        }
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxWidth(),
            contentAlignment = Alignment.TopCenter
        ) {
            ContentView(navController)
        }
    }
}

@Composable
private fun ContentView(
    navController: NavController,
    modifier: Modifier = Modifier,
    vm: SettingViewModel = viewModel()
) {
    val state by vm.state.collectAsState()
    Column(
        modifier = modifier
            .verticalScroll(rememberScrollState())
            .padding(10.dp)
            .widthIn(max = 360.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Spacer(Modifier.height(10.dp))
        Image(
            painter = painterResource(R.drawable.logo_xs),
            contentDescription = null,
            modifier = Modifier
                .size(70.dp)
                .clip(RoundedCornerShape(12.dp))
        )
        Spacer(Modifier.height(20.dp))
        TextEditBox(
            value = state.npcSetting.host,
            placeholder = "服务器",
            onValueChange = { host ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(host = host)
                    )
                }
            },
            icon = Icons.Default.Storage,
        )
        Spacer(Modifier.height(10.dp))

        TextEditBox(
            value = state.npcSetting.tcpPort,
            placeholder = "TCP端口(默认1881)",
            onValueChange = { tcpPort ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(tcpPort = tcpPort)
                    )
                }
            },
            icon = Icons.Default.Pin,
        )
        Spacer(Modifier.height(10.dp))

        TextEditBox(
            value = state.npcSetting.udpPort,
            placeholder = "UDP端口(默认1882)",
            onValueChange = { udpPort ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(udpPort = udpPort)
                    )
                }
            },
            icon = Icons.Default.Pin,
        )

        Spacer(Modifier.height(20.dp))

        TextEditBox(
            value = state.npcSetting.key,
            placeholder = "连接秘钥",
            onValueChange = { key ->
                vm.update {
                    state.copy(
                        npcSetting = state.npcSetting.copy(key = key)
                    )
                }
            },
            icon = Icons.Default.Key,
        )

        Spacer(Modifier.height(20.dp))

        Button(
            onClick = {
                vm.save {
                    navController.relaunch("home")
                }
            },
            modifier = Modifier.fillMaxWidth(),
            colors = ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.extraColors.bgPrimary,      // 按钮背景色
                contentColor = Color.White        // 文字颜色
            ),
            shape = RoundedCornerShape(6.dp)    // 圆角大小
        ) {
            Text("保存并连接")
        }
    }
}


/**
 * 文本输入框
 */
@Composable
private fun TextEditBox(
    modifier: Modifier = Modifier,
    value: String,
    placeholder: String = "",
    icon: ImageVector,
    onValueChange: (String) -> Unit,
) = Row(
    modifier = modifier
        .border(
            width = 1.dp,
            color = MaterialTheme.extraColors.bgPrimary,
            shape = RoundedCornerShape(6.dp)
        )
        .padding(vertical = 12.dp, horizontal = 5.dp),
    verticalAlignment = Alignment.CenterVertically
) {
    Icon(
        imageVector = icon,
        contentDescription = null,
        tint = MaterialTheme.colorScheme.secondary,
        modifier = Modifier.size(16.dp)
    )
    Spacer(Modifier.width(8.dp))
    Box(modifier = Modifier.fillMaxWidth()) {
        if (value.isEmpty()) {
            Text(
                placeholder,
                modifier = Modifier.fillMaxWidth(),
                color = MaterialTheme.colorScheme.secondary
            )
        }
        BasicTextField(
            modifier = Modifier.fillMaxWidth(),
            value = value,
            onValueChange = onValueChange,
            singleLine = true,
        )
    }
}
