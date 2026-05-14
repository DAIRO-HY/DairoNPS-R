package cn.dairo.npc.page.home

import android.app.Application
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material.icons.filled.WifiOff
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import cn.dairo.npc.NPCRustBridge
import cn.dairo.npc.ThisApplication
import cn.dairo.npc.bean.NpcStatus
import cn.dairo.npc.extension.readableSize
import cn.dairo.npc.repository.NpcRepository
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class HomeViewModel(private val application: Application) : AndroidViewModel(application) {

    /**
     * 记录上次获取到的入网流量,用来统计网速
     */
    private var lastInLen = 0L

    /**
     * 记录上次获取到的出网流量,用来统计网速
     */
    private var lastOutLen = 0L

    private val _state = MutableStateFlow(
        HomeState()
    )
    val state = this._state.asStateFlow()

    private var loopGetStatusJob: Job? = null

    init {
        viewModelScope.launch {
            val repo = NpcRepository(application)
            _state.update {
                it.copy(
                    npcSetting = repo.loadSetting(),//配置信息
                    npcInfo = NPCRustBridge.getInfo(),//客户端信息
                    npcStatus = NpcStatus(),
                    isOpened = repo.isOpened(),
                )
            }
            if (repo.isOpened()) {
                openNpc()
            }
        }
    }

    /**
     * 轮询获取NPC状态
     */
    fun loopGetStatus() {
        this.loopGetStatusJob?.cancel()
        this.loopGetStatusJob = viewModelScope.launch {
            while (true) {
                updateStatus()
                delay(1000)
            }
        }
    }

    /**
     * 停止循环获取状态
     */
    fun cancelLoopGetStatus() {
        this.loopGetStatusJob?.cancel()
        this.lastInLen = 0L
        this.lastOutLen = 0L
    }

    private fun updateStatus() {
        val npcStatus = NPCRustBridge.getStatus()
        _state.update {
            val inSpeed = if (lastInLen == 0L) {
                0
            } else {
                npcStatus.inLen - lastInLen
            }
            val outSpeed = if (lastOutLen == 0L) {
                0
            } else {
                npcStatus.outLen - lastOutLen
            }
            lastInLen = npcStatus.inLen
            lastOutLen = npcStatus.outLen
            it.copy(
                npcStatus = npcStatus,
                statusLabel = if (npcStatus.isRunning) "⚫\uFE0E连接正常" else "⚫\uFE0E连接断开",
                statusColor = if (npcStatus.isRunning) ThisApplication.colorScheme.primary else ThisApplication.colorScheme.error,
                statusIcon = if (npcStatus.isRunning) Icons.Default.Wifi else Icons.Default.WifiOff,
                inSpeed = inSpeed.readableSize + "/s",
                outSpeed = outSpeed.readableSize + "/s",
            )
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
                NPCRustBridge.close()
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
        NPCRustBridge.open(
            npcSetting.host,
            npcSetting.tcpPort.toShort(),
            npcSetting.udpPort.toShort(),
            npcSetting.key
        )
    }
}