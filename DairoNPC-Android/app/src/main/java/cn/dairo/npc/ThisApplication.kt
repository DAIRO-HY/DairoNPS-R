package cn.dairo.npc

import android.app.Application

/**
 * 全局Application对象
 */
class ThisApplication: Application() {
    override fun onCreate() {
        super.onCreate()
        app = this
    }

    companion object{

        /**
         * 全局保存Application对象
         */
        lateinit var app:Application
    }
}