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
                    npc = repo.loadSetting()
                )
            }
        }
    }
    fun save(block:()-> Unit) {
        val npc = this.state.value.npc
        if(npc.host.isBlank()){
            application.toast("服务器地址不能为空")
            return
        }

        if(npc.key.isBlank()){
            application.toast("秘钥不能为空")
            return
        }
        if(npc.tcpPort.toShortOrNull() == null || npc.tcpPort.toShort() <= 0) {
            application.toast("TCP端口必须为正整数且小于65536")
            return
        }
        if(npc.udpPort.toShortOrNull() == null || npc.udpPort.toShort() <= 0) {
            application.toast("UDP端口必须为正整数且小于65536")
            return
        }
        viewModelScope.launch {
            repo.saveSetting(
                _state.value.npc
            )
            repo.saveSet(true)
            block()
        }
    }
}