package cn.dairo.npc.activity

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.content.res.ColorStateList
import android.graphics.Color
import android.os.Bundle
import android.view.View
import android.widget.TextView
import android.widget.Button
import android.widget.EditText
import cn.dairo.npc.Constant
import cn.dairo.npc.R
import cn.dairo.npc.RustBridge
import kotlin.concurrent.thread

/**
 * 客户端编辑页面
 */
class HomeActivity : Activity() {

    /**
     * 页面是否隐藏了,如果隐藏了就不更新状态了
     */
    private var updateStatusThread: Thread? = null

    /**
     * 连接NPC开关
     */
    private val btnToggle by lazy { this.findViewById<Button>(R.id.btnToggle) }

    /**
     * 变更配置按钮
     */
    private val btnEditNps by lazy { this.findViewById<Button>(R.id.btnEditNps) }

    /**
     * 连接状态图标
     */
    private val icoStatus by lazy { this.findViewById<View>(R.id.icoStatus) }

    /**
     * 连接状态
     */
    private val txtIsRunningLabel by lazy { this.findViewById<TextView>(R.id.txtIsRunningLabel) }

    /**
     * 连接状态信息
     */
    private val txtIsRunningMsg by lazy { this.findViewById<TextView>(R.id.txtIsRunningMsg) }

    /**
     * 桥接数量
     */
    private val txtBridgeCount by lazy { this.findViewById<TextView>(R.id.txtBridgeCount) }

    /**
     * 连接池数量
     */
    private val txtPoolCount by lazy { this.findViewById<TextView>(R.id.txtPoolCount) }

    /**
     * NPC客户端ID
     */
    private val txtNpcClientId by lazy { this.findViewById<TextView>(R.id.txtNpcClientId) }

    /**
     * NPC版本号
     */
    private val txtNpcVersion by lazy { this.findViewById<TextView>(R.id.txtNpcVersion) }

    /**
     * 服务器
     */
    private val txtHost by lazy { this.findViewById<TextView>(R.id.txtHost) }

    /**
     * tcp端口
     */
    private val txtTcpPort by lazy { this.findViewById<TextView>(R.id.txtTcpPort) }

    /**
     * Udp端口
     */
    private val txtUdpPort by lazy { this.findViewById<TextView>(R.id.txtUdpPort) }

    /**
     * 秘钥
     */
    private val txtKey by lazy { this.findViewById<TextView>(R.id.txtKey) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        super.actionBar?.hide()
        setContentView(R.layout.activity_home)
        initView()
        if (this.getSharedPreferences(Constant.NPS_SHARED_PREFERENCES_NAME, Context.MODE_PRIVATE)
                .getBoolean("is_opened", false)
        ) {
            this.openNpc()
        }
    }

    override fun onStart() {
        super.onStart()
        this.showStatus()
    }

    private fun initView() {
        val sharedPreferences = this.getSharedPreferences(Constant.NPS_SHARED_PREFERENCES_NAME,Context.MODE_PRIVATE)
        this.txtHost.setText(sharedPreferences.getString("host",""))
        this.txtTcpPort.setText(sharedPreferences.getString("tcp_port",""))
        this.txtUdpPort.setText(sharedPreferences.getString("udp_port",""))
        this.txtKey.setText(sharedPreferences.getString("key",""))


        this.btnToggle.setOnClickListener(this::onOpenNpcClick)
        this.btnEditNps.setOnClickListener{

            //重置输入状态
            this.getSharedPreferences(Constant.NPS_SHARED_PREFERENCES_NAME, Context.MODE_PRIVATE)
                .edit().putBoolean("is_inputted",false).apply()
            val intent = Intent(this, EditNpsActivity::class.java)
            this.startActivity(intent)
            this.finish()
        }

        val npcInfo = RustBridge.getNpcInfo()
        this.txtNpcClientId.text = if(npcInfo.clientId == 0L) "未连接" else npcInfo.clientId.toString()
        this.txtNpcVersion.text = npcInfo.version
    }

    private fun showStatus() {
        this.updateStatusThread?.interrupt()
        this.updateStatusThread = thread {
            while (true) {
                try {
                    updateStatus()
                    Thread.sleep(1000)
                } catch (_: Exception) {
                    break
                }
            }
        }
    }

    private fun updateStatus() {
        val info = RustBridge.getStatusInfo()
        this.runOnUiThread {
            this.txtIsRunningMsg.text = info.connectMsg
            this.txtBridgeCount.text = info.bridgeCount.toString()
            this.txtPoolCount.text = info.poolCount.toString()
            this.btnToggle.text = if (info.isOpened) "断开连接" else "启动连接"
            this.btnToggle.tag = info.isOpened

            if (info.isRunning) {
                val color = Color.parseColor("#198754")
                this.icoStatus.setBackgroundResource(R.drawable.bi_wifi)
                this.icoStatus.backgroundTintList = ColorStateList.valueOf(color)
                this.txtIsRunningLabel.text = "⚫\uFE0E连接正常"
                this.txtIsRunningLabel.setTextColor(color)
                this.txtIsRunningMsg.setTextColor(color)
            } else {
                val color = Color.parseColor("#DC3545")
                this.icoStatus.setBackgroundResource(R.drawable.bi_wifi_off)
                this.icoStatus.backgroundTintList = ColorStateList.valueOf(color)
                this.txtIsRunningLabel.text = "⚫\uFE0E连接断开"
                this.txtIsRunningLabel.setTextColor(color)
                this.txtIsRunningMsg.setTextColor(color)
            }
        }
    }

    /**
     * 打开/关闭NPC服务
     */
    private fun onOpenNpcClick(v: View) {
        val spEdit =
            this.getSharedPreferences(Constant.NPS_SHARED_PREFERENCES_NAME, Context.MODE_PRIVATE)
                .edit()
        if (v.tag as Boolean) {//关闭NPC
            spEdit.putBoolean("is_opened", false)
            RustBridge.stop()
        } else {//打开NPC
            spEdit.putBoolean("is_opened", true)
            this.openNpc()
        }
        spEdit.apply()
    }

    private fun openNpc() {
        val sharedPreferences =
            this.getSharedPreferences(Constant.NPS_SHARED_PREFERENCES_NAME, Context.MODE_PRIVATE)
        val host = sharedPreferences.getString("host", "")!!
        val tcpPort = sharedPreferences.getString("tcp_port", "")!!.toShort()
        val ucpPort = sharedPreferences.getString("udp_port", "")!!.toShort()
        val key = sharedPreferences.getString("key", "")!!
        thread {
            RustBridge.start(host, tcpPort, ucpPort, key)
        }
    }

    override fun onPause() {
        super.onPause()
        this.updateStatusThread?.interrupt()
    }
}