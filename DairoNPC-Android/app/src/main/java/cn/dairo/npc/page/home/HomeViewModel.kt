package cn.dairo.npc.page.home

import android.app.Application
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material.icons.filled.WifiOff
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import cn.dairo.npc.RustBridge
import cn.dairo.npc.ThisApplication
import cn.dairo.npc.bean.NpcStatus
import cn.dairo.npc.repository.NpcRepository
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlin.concurrent.thread

class HomeViewModel(private val application: Application) : AndroidViewModel(application) {

    private val _state = MutableStateFlow(
        HomeState()
    )
    val state = this._state.asStateFlow()

    init {
        viewModelScope.launch {
            _state.update {
                val repo = NpcRepository(application)
                it.copy(
                    npcSetting = repo.loadSetting(),//配置信息
                    npcInfo = RustBridge.getInfo(),//客户端信息
                    npcStatus = NpcStatus(),
                    isOpened = repo.isOpened(),
                )
            }
        }
        loopGetStatus()
    }

    /**
     * 轮询获取NPC状态
     */
    private fun loopGetStatus() {
        viewModelScope.launch {
            while (true) {
                _state.update {
                    val status = RustBridge.getStatus()
                    it.copy(
                        npcStatus = status,
                        statusLabel = if (status.isRunning) "⚫\uFE0E连接正常" else "⚫\uFE0E连接断开",
                        statusColor = if (status.isRunning) ThisApplication.colorScheme.primary else ThisApplication.colorScheme.error,
                        statusIcon = if (status.isRunning) Icons.Default.Wifi else Icons.Default.WifiOff,
                    )
                }
                delay(1000)
            }
        }
    }

    /**
     * 标记为重新配置
     */
    fun reset(block: () -> Unit) {
        viewModelScope.launch {
            NpcRepository(application).saveSet(false)
            block()
        }
    }

    /**
     * 打开/关闭NPC服务
     */
    fun onOpenNpcClick() {
        viewModelScope.launch {
            val repo = NpcRepository(application)
            if (repo.isOpened()) {//关闭NPC
                repo.setOpened(false)
                RustBridge.stop()
            } else {//打开NPC
                repo.setOpened(true)
                openNpc()
            }
            _state.update {
                it.copy(
                    isOpened = repo.isOpened(),
                )
            }
        }
    }

    private fun openNpc() {
        val npcSetting = this.state.value.npcSetting
        GlobalScope.launch {
            RustBridge.start(
                npcSetting.host,
                npcSetting.tcpPort.toShort(),
                npcSetting.udpPort.toShort(),
                npcSetting.key
            )
        }
    }
}