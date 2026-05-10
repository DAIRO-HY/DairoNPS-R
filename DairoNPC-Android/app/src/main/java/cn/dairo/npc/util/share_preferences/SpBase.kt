package cn.dairo.npc.util.share_preferences

import android.content.Context
import android.content.SharedPreferences
import cn.dairo.npc.ThisApplication

class SpBase(sharedName: String) {
    private val sharedPreferences: SharedPreferences

    init {
        this.sharedPreferences = ThisApplication.Companion.app.getSharedPreferences(
            sharedName, Context.MODE_PRIVATE
        )
    }

    /**
     * 保存数据
     * @param key
     * @param value
     * @return
     */
    fun put(
        key: String,
        value: Any?
    ): Boolean {
        if (value == null) {
            return this.sharedPreferences.edit().remove(key).commit()
        }
        val edit = this.sharedPreferences.edit()
        when (value) {
            is Boolean -> edit.putBoolean(key, value)
            is Int -> edit.putInt(key,value)
            is Long -> edit.putLong(key,value)
            is Float -> edit.putFloat(key,value)
            is String -> edit.putString(key, value as String?)
            else -> edit.putString(key, value.toString())
        }
        return edit.commit()
    }

    /**
     * 移除某个key
     */
    fun delete(key: String): Boolean {
        return sharedPreferences.edit().remove(key).commit()
    }

    /**
     * 清除整个配置文件
     */
    fun clear(): Boolean {
        return sharedPreferences.edit().clear().commit()
    }

    fun getString(key: String): String? {
        return sharedPreferences.getString(key, null)
    }

    fun getString(key: String, defaultVal: String): String {
        return sharedPreferences.getString(key, defaultVal)!!
    }

    fun getBoolean(key: String): Boolean {
        return sharedPreferences.getBoolean(key, false)
    }

    fun getBoolean(key: String, defaultVal: Boolean): Boolean {
        return sharedPreferences.getBoolean(key, defaultVal)
    }

    fun getInt(key: String): Int {
        return getInt(key, 0)
    }

    fun getInt(key: String, dVal: Int): Int {
        return sharedPreferences.getInt(key, dVal)
    }

    fun getLong(key: String): Long {
        return sharedPreferences.getLong(key, 0)
    }

    fun getFloat(key: String): Float {
        return sharedPreferences.getFloat(key, 0f)
    }

    fun getFloat(key: String, dVal: Float): Float {
        return sharedPreferences.getFloat(key, dVal)
    }

    //    public float getDouble(String key) {
    //        return mShared.getFloat(key, 0F);
    //    }
    //
    //    public float getDouble(String key,float dVal) {
    //        return mShared.(key, dVal);
    //    }
    val all: Map<String, *>

        /**
         * @return 获取全部
         */
        get() = sharedPreferences.all
}
