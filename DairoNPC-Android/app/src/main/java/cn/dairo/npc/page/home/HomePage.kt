package cn.dairo.npc.page.home

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.HotTub
import androidx.compose.material.icons.filled.SyncAlt
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import cn.dairo.npc.extension.relaunch

@Composable
fun HomePage(
    navController: NavController, modifier: Modifier = Modifier, vm: HomeViewModel = viewModel()
) {
    val context = LocalContext.current
    val state by vm.state.collectAsState()
    Column(modifier = modifier
        .verticalScroll(rememberScrollState())
        .padding(10.dp)) {
        NpcStatusView()
        Spacer(Modifier.height(10.dp))
        DataSizeView()
        Spacer(Modifier.height(10.dp))
        ConnectCountView()
        Spacer(Modifier.height(30.dp))
        Button(
            modifier = Modifier
                .fillMaxWidth()
                .height(40.dp),
            contentPadding = PaddingValues(0.dp),
            onClick = {
                vm.reset {
                    navController.relaunch("setting")
                }
            }
        ) {
            Text("变更配置", fontSize = 14.sp)
        }
    }
}

/**
 * NPC信息标签
 */
@Composable
private fun NpcInfoView(title: String, value: String) = Row {
    Text(title, fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
    Text(
        value,
        modifier = Modifier
            .weight(1f),
        fontSize = 14.sp,
        textAlign = TextAlign.End
    )
}

@Composable
private fun NpcStatusView(vm: HomeViewModel = viewModel()) = Column {
    val state by vm.state.collectAsState()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)//子控件fillMaxHeight()使其子控件一样高
            .clip(RoundedCornerShape(8.dp))
            .background(MaterialTheme.colorScheme.surfaceContainerHigh),
    ) {
        Column(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
                .padding(10.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally//内部控件横向居中
        ) {
            Icon(
                imageVector = state.statusIcon,
                contentDescription = null,
                tint = state.statusColor,
                modifier = Modifier.size(60.dp)
            )
            Spacer(Modifier.height(20.dp))
            Button(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(30.dp),
                contentPadding = PaddingValues(0.dp),
                onClick = vm::onOpenNpcClick
            ) {
                Text(if (state.isOpened) "断开连接" else "启动连接", fontSize = 12.sp)
            }
        }
        Column(
            modifier = Modifier
                .weight(2f)
                .fillMaxHeight()
                .padding(10.dp)
        ) {
            Text(state.statusLabel, fontSize = 26.sp, color = state.statusColor)
            Text(state.npcStatus.connectMsg, fontSize = 14.sp)
            NpcInfoView("服务器", state.npcSetting.host)
            NpcInfoView("TCP端口", state.npcSetting.tcpPort)
            NpcInfoView("UDP端口", state.npcSetting.udpPort)
            NpcInfoView("连接秘钥", state.npcSetting.key)
            NpcInfoView("NPC客户端ID", state.npcInfo.clientId.toString())
            NpcInfoView("NPC版本号", state.npcInfo.version)
        }
    }
}

@Composable
private fun DataSizeView(vm: HomeViewModel = viewModel()) = Column(
    modifier = Modifier
        .fillMaxWidth()
        .height(IntrinsicSize.Min)//子控件fillMaxHeight()使其子控件一样高
        .clip(RoundedCornerShape(8.dp))
        .background(MaterialTheme.colorScheme.surfaceContainerHigh)
        .padding(10.dp),
) {
    Text("流量统计", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
    Spacer(modifier = Modifier.height(10.dp))
    Row(
        modifier = Modifier
            .fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,//让子控件垂直居中
    ) {
        Icon(
            imageVector = Icons.Default.ArrowUpward,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
            modifier = Modifier.size(30.dp)
        )
        Column(
            modifier = Modifier
                .weight(1f),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text("上行流量", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
            Row {
                Text("12.345", fontSize = 16.sp)
                Text("MB", fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
            }
            Text("1.23MB/s", fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
        }
        VerticalDivider(
            modifier = Modifier.padding(15.dp),
            thickness = 1.dp,
            color = Color.Gray
        )
        Column(
            modifier = Modifier
                .weight(1f),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text("下行流量", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
            Row {
                Text("12.345", fontSize = 16.sp)
                Text("MB", fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
            }
            Text("1.23MB/s", fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
        }
        Icon(
            imageVector = Icons.Default.ArrowDownward,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
            modifier = Modifier.size(30.dp)
        )
    }
}

/**
 * 桥接ji连接池数量信息
 */
@Composable
private fun ConnectCountView(vm: HomeViewModel = viewModel()) = Column(
    modifier = Modifier
        .fillMaxWidth()
        .height(IntrinsicSize.Min)//子控件fillMaxHeight()使其子控件一样高
        .clip(RoundedCornerShape(8.dp))
        .background(MaterialTheme.colorScheme.surfaceContainerHigh)
        .padding(10.dp),
) {
    val state by vm.state.collectAsState()

    Text(text = "连接信息", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
    Spacer(modifier = Modifier.height(10.dp))
    Row(
        modifier = Modifier
            .fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,//让子控件垂直居中
    ) {
        Icon(
            imageVector = Icons.Default.SyncAlt,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
            modifier = Modifier.size(30.dp)
        )
        Column(
            modifier = Modifier
                .weight(1f),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text("桥接数", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
            Text(state.npcStatus.bridgeCount.toString(), fontSize = 16.sp)
        }
        VerticalDivider(
            modifier = Modifier.padding(15.dp),
            thickness = 1.dp,
            color = Color.Gray
        )
        Column(
            modifier = Modifier.weight(1f),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text("连接池", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
            Text(state.npcStatus.poolCount.toString(), fontSize = 16.sp)
        }
        Icon(
            imageVector = Icons.Default.HotTub,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
            modifier = Modifier.size(30.dp)
        )
    }
}


@Preview(showBackground = true)
@Composable
fun HomePagePreview() {
    _root_ide_package_.cn.dairo.npc.ui.theme.MyApplicationTheme {
        val navController = rememberNavController()
        NavHost(
            navController = navController,
            startDestination = "npc-config"
        ) {
            composable("npc-config") { _ ->
                HomePage(navController)
            }
        }
    }
}