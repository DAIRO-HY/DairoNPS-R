package cn.dairo.npc.activity

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.view.View
import android.widget.Button
import cn.dairo.npc.R
import cn.dairo.npc.RustBridge
import cn.dairo.npc.util.Toast
import java.net.Socket
import kotlin.concurrent.thread

/**
 * 客户端编辑页面
 */
class ClientEditActivity : Activity() {

    /**
     * 保存按钮
     */
    private val btnConnect = lazy { this.findViewById<Button>(R.id.btnConnect) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_login)
        initView()
    }

    private fun initView() {
        this.findViewById<View>(R.id.btnStart).setOnClickListener(this::onStartNpc)
        this.findViewById<View>(R.id.btnStop).setOnClickListener(this::onStopNpc)
        this.findViewById<View>(R.id.btnConnect).setOnClickListener(this::onSaveClick)
    }

    /**
     * 保存点击事件
     */
    private fun onSaveClick(v: View) {
        val intent = Intent(this, HomeActivity::class.java)
        this.startActivity(intent)
        this.finish()
    }

    /**
     * 登录
     */
    private fun onStartNpc(v: View) {
        thread{

//            val socket = Socket("192.168.3.57",1881)
//            val oStream = socket.getOutputStream()
//            oStream.write(0)

            val test = RustBridge.start("192.168.3.57",1881,1882,"njeHds*fs4tfsd")
            println(test)
        }
    }

    /**
     * 停止
     */
    private fun onStopNpc(v: View) {
//        RustBridge.stop()
//        val obj = RustBridge.getStatusInfo()
//        Toast.show(obj.toString())

        val rs = RustBridge.getHello("Test")
        println(rs)
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