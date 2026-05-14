package cn.dairo.npc.repository

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import cn.dairo.npc.bean.NpcSetting
import cn.dairo.npc.extension.npcDataStore
import com.google.gson.Gson
import kotlinx.coroutines.flow.first

class NpcRepository(private val context: Context) {

    /**
     * 配置存储key
     */
    private val settingKey = stringPreferencesKey("setting")

    /**
     * 标记是否是打开状态的
     */
    private val isOpenedKey = booleanPreferencesKey("isOpened")

    /**
     * 是否已经配置标记存储key
     */
    private val isSetKey = booleanPreferencesKey("isSet")

    /**
     * 从缓存加载
     */
    suspend fun loadSetting(): NpcSetting {
        val pref = this.context.npcDataStore.data.first()
        val json = pref[settingKey]?:return NpcSetting()
        return Gson().fromJson(json, NpcSetting::class.java)
    }

    /**
     * 保存配置
     */
    suspend fun saveSetting(bean: NpcSetting){
        this.context.npcDataStore.edit {
            it[settingKey] = Gson().toJson(bean)
        }
    }

    /**
     * 获取标记是否是打开状态
     */
    suspend fun isOpened(): Boolean {
        val pref = this.context.npcDataStore.data.first()
        return pref[isOpenedKey] ?: false
    }

    /**
     * 设置标记是否是打开状态
     */
    suspend fun setOpened(flag:Boolean) {
        this.context.npcDataStore.edit {
            it[isOpenedKey] = flag
        }
    }

    /**
     * 获取是否已经设置完成
     */
    suspend fun isSet(): Boolean {
        val pref = this.context.npcDataStore.data.first()
        return pref[isSetKey] ?: false
    }

    /**
     * 获取是否已经设置完成
     */
    suspend fun saveSet(flag:Boolean) {
        this.context.npcDataStore.edit {
            it[isSetKey] = flag
        }
    }

}