package cn.dairo.npc.page.setting

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import cn.dairo.npc.extension.toast
import cn.dairo.npc.repository.NpcRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class SettingViewModel(private val application: Application) : AndroidViewModel(application) {

    private val repo = NpcRepository(application)

    private val _state = MutableStateFlow(
        SettingState()
    )
    val state = this._state.asStateFlow()
    init {
        load()
    }

    /**
     * 暴露更新函数
     */
    fun update(block: (SettingState) -> SettingState){
        this._state.update(block)
    }

    private fun load() {
        viewModelScope.launch {
            _state.update {
                it.copy(
                    npcSetting = repo.loadSetting()
                )
            }
        }
    }
    fun save(block:()-> Unit) {
        var npcSetting = this.state.value.npcSetting
        if(npcSetting.tcpPort.isBlank()){
            npcSetting = npcSetting.copy(tcpPort = "1881")
        }
        if(npcSetting.udpPort.isBlank()){
            npcSetting = npcSetting.copy(udpPort = "1882")
        }

        if(npcSetting.host.isBlank()){
            application.toast("服务器地址不能为空")
            return
        }
        if(npcSetting.key.isBlank()){
            application.toast("秘钥不能为空")
            return
        }
        if(npcSetting.tcpPort.toShortOrNull() == null || npcSetting.tcpPort.toShort() <= 0) {
            application.toast("TCP端口必须为正整数且小于65536")
            return
        }
        if(npcSetting.udpPort.toShortOrNull() == null || npcSetting.udpPort.toShort() <= 0) {
            application.toast("UDP端口必须为正整数且小于65536")
            return
        }
        viewModelScope.launch {
            repo.saveSetting(npcSetting)
            repo.saveSet(true)
            repo.setOpened(true)
            block()
        }
    }
}