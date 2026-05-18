package cn.dairo.npc.page.home

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
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
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.DynamicFeed
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.SyncAlt
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavController
import cn.dairo.npc.extension.readableSize
import cn.dairo.npc.extension.relaunch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomePage(
    navController: NavController, modifier: Modifier = Modifier, vm: HomeViewModel = viewModel()
) {
    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_PAUSE -> {
                    // 用户返回桌面
                    vm.cancelLoopGetStatus()
                }

                Lifecycle.Event.ON_RESUME -> {
                    vm.loopGetStatus()
                }

                else -> {}
            }
        }

        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                modifier = modifier.background(Color.Red),
                title = {
                    Text("DairoNPC", color = MaterialTheme.colorScheme.onPrimary)
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primary
                ),
                actions = {
                    IconButton(onClick = {
                        vm.onResetClick {
                            navController.relaunch("setting")
                        }
                    }) {
                        Icon(
                            imageVector = Icons.Default.Settings,
                            tint = MaterialTheme.colorScheme.onPrimary,
                            contentDescription = null
                        )
                    }
                }
            )
        }
    ) { innerPadding ->
        ContentView(Modifier.padding(innerPadding))
    }


}

@Composable
private fun ContentView(modifier: Modifier = Modifier, vm: HomeViewModel = viewModel()) {
    Column(
        modifier = modifier
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 15.dp)
    ) {
        Spacer(Modifier.height(20.dp))
        NpcStatusView()
        Spacer(Modifier.height(20.dp))
        DataSizeView()
        Spacer(Modifier.height(20.dp))
        ConnectCountView()
        Spacer(Modifier.height(20.dp))
        SystemInfoView()
        Spacer(Modifier.height(20.dp))
    }
}

@Composable
fun CardView(content: @Composable BoxScope.() -> Unit) {
//    Box(
//        modifier = Modifier
//            .clip(RoundedCornerShape(8.dp))
//            .shadow(
//                elevation = 15.dp,
//                shape = RoundedCornerShape(16.dp),
//                clip = true
//            )
////            .background(MaterialTheme.colorScheme.surfaceContainerHigh),
//            .background(Color.Black),
//    ) {
//        content()
//    }

    Card(
        elevation = CardDefaults.cardElevation(
            defaultElevation = 15.dp
        )
    ) {
        Box {
            content()
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
private fun NpcStatusView(vm: HomeViewModel = viewModel()) = CardView {
    val state by vm.state.collectAsState()
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)//子控件fillMaxHeight()使其子控件一样高
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
        }
    }
}

@Composable
private fun DataSizeView(vm: HomeViewModel = viewModel()) = CardView {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)//子控件fillMaxHeight()使其子控件一样高
            .padding(10.dp),
    ) {
        val state by vm.state.collectAsState()
        Text("流量统计", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
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
                Text(state.npcStatus.outLen.readableSize, fontSize = 14.sp)
                Text(state.outSpeed, fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
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
                Text(state.npcStatus.inLen.readableSize, fontSize = 14.sp)

                Text(state.inSpeed, fontSize = 12.sp, color = MaterialTheme.colorScheme.secondary)
            }
            Icon(
                imageVector = Icons.Default.ArrowDownward,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(30.dp)
            )
        }
    }
}

/**
 * 桥接连接池数量信息
 */
@Composable
private fun ConnectCountView(vm: HomeViewModel = viewModel()) = CardView {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)//子控件fillMaxHeight()使其子控件一样高
            .padding(10.dp),
    ) {
        val state by vm.state.collectAsState()

        Text(text = "连接数量", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
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
                imageVector = Icons.Default.DynamicFeed,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(30.dp)
            )
        }
    }
}


/**
 * 系统信息
 */
@Composable
private fun SystemInfoView(vm: HomeViewModel = viewModel()) = CardView {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)//子控件fillMaxHeight()使其子控件一样高
            .padding(10.dp),
    ) {
        val state by vm.state.collectAsState()

        Text(text = "系统信息", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
        Row(
            modifier = Modifier
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,//让子控件垂直居中
        ) {
            Column(
                modifier = Modifier
                    .weight(1f),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text("NPC版本", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
                Text(state.npcInfo.version, fontSize = 12.sp)
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
                Text("客户端ID", fontSize = 14.sp, color = MaterialTheme.colorScheme.secondary)
                Text(state.npcInfo.clientId.toString(), fontSize = 12.sp)
            }
        }
    }
}


@OptIn(ExperimentalMaterial3Api::class)
@Preview(showBackground = true)
@Composable
fun HomePagePreview() {
    NpcInfoView("测试", "123")
}