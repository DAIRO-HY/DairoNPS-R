package cn.dairo.npc.activity

import android.app.Activity
import android.os.Bundle
import android.widget.TextView
import cn.dairo.npc.R
import cn.dairo.npc.RustBridge
import cn.dairo.npc.util.Toast
import kotlin.concurrent.thread

/**
 * 客户端编辑页面
 */
class HomeActivity : Activity() {

    /**
     * 桥接数量
     */
    private val txtBridgeCount by lazy { this.findViewById<TextView>(R.id.txtBridgeCount) }

    /**
     * 连接池数量
     */
    private val txtPoolCount by lazy { this.findViewById<TextView>(R.id.txtPoolCount) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        super.actionBar?.hide()
        setContentView(R.layout.activity_home)
        initView()
        showStatus()
    }

    private fun initView() {
//        this.bind.btnStart.setOnClickListener {
//            this.onStartNpc()
//        }
//        this.bind.btnStop.setOnClickListener {
//            this.onStopNpc()
//        }
//        this.findViewById<View>(R.id.btnConnect).setOnClickListener {
//            this.onStartNpc()
//        }
    }

    private fun showStatus(){
        thread {
            while (true){
                Thread.sleep(1000)
                val info = RustBridge.getStatusInfo()
                this.runOnUiThread {
                    this.txtBridgeCount.text = info.bridgeCount.toString()
                    this.txtPoolCount.text = info.poolCount.toString()
                }
            }
        }
    }

    /**
     * 登录
     */
    private fun onStartNpc() {
        thread {
            val test = RustBridge.start("192.168.3.57",1881,1882,"njeHds*fs4tfsd")
            println(test)
        }
    }

    /**
     * 停止
     */
    private fun onStopNpc() {
//        RustBridge.stop()
        val obj = RustBridge.getStatusInfo()
        Toast.show(obj.toString())

        val rs = RustBridge.getHello("Test")
        Toast.show(rs)
    }


    companion object {

        /**
         * 退出登录
         */
        fun logout(activity: Activity) {
        }
    }
}