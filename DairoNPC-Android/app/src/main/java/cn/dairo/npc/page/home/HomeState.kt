package cn.dairo.npc.page.home

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import cn.dairo.npc.bean.NpcInfo
import cn.dairo.npc.bean.NpcStatus
import cn.dairo.npc.bean.NpcSetting

data class HomeState(
    //NPC客户端配置信息
    val npcSetting: NpcSetting = NpcSetting(),

    //NPC客户端信息
    val npcInfo: NpcInfo = NpcInfo(),

    //NPC连接信息
    val npcStatus:NpcStatus = NpcStatus(),

    //状态标签文字
    val statusIcon:ImageVector = Icons.Default.Wifi,

    //状态标签文字
    val statusLabel:String = "",

    //标记是否已经打开过
    val isOpened:Boolean = false,

    //状态颜色
    val statusColor:Color = Color.Gray,

    //入网网速
    val inSpeed:String = "",

    //出网网速
    val outSpeed:String = ""
)