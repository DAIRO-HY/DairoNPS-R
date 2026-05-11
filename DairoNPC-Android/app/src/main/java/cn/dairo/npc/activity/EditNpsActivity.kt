package cn.dairo.npc.activity

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.view.View
import android.widget.EditText
import cn.dairo.npc.Constant
import cn.dairo.npc.R
import cn.dairo.npc.util.Toast

/**
 * 客户端编辑页面
 */
class EditNpsActivity : Activity() {

    /**
     * 数据持久换
     */
    private val sharedPreferences by lazy{
        this.getSharedPreferences(Constant.NPS_SHARED_PREFERENCES_NAME,Context.MODE_PRIVATE)
    }


    /**
     * 服务器
     */
    private val editHost by lazy { this.findViewById<EditText>(R.id.editHost) }

    /**
     * tcp端口
     */
    private val editTcpPort by lazy { this.findViewById<EditText>(R.id.editTcpPort) }

    /**
     * Udp端口
     */
    private val editUdpPort by lazy { this.findViewById<EditText>(R.id.editUdpPort) }

    /**
     * 秘钥
     */
    private val editKey by lazy { this.findViewById<EditText>(R.id.editKey) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        //如果已经输入过了，就直接打开主页
        if (this.getSharedPreferences(Constant.NPS_SHARED_PREFERENCES_NAME, Context.MODE_PRIVATE)
                .getBoolean("is_inputted", false)
        ) {
            this.toHome()
            return
        }
        setContentView(R.layout.activity_edit_nps)
        initView()
    }

    private fun initView() {
        this.editHost.setText(sharedPreferences.getString("host",""))
        this.editTcpPort.setText(sharedPreferences.getString("tcp_port",""))
        this.editUdpPort.setText(sharedPreferences.getString("udp_port",""))
        this.editKey.setText(sharedPreferences.getString("key",""))

        this.findViewById<View>(R.id.btnSaveAndOpen).setOnClickListener(this::onSaveClick)
    }

    /**
     * 保存点击事件
     */
    private fun onSaveClick(v: View) {

        val host = this.editHost.text.toString()
        val tcpPort = this.editTcpPort.text.toString().ifEmpty { "1881" }
        val udpPort = this.editUdpPort.text.toString().ifEmpty { "1882" }
        val key = this.editKey.text.toString()

        if(host.isBlank()){
            Toast.show("服务器地址不能为空")
            return
        }

        if(key.isBlank()){
            Toast.show("秘钥不能为空")
            return
        }
        if(tcpPort.toShortOrNull() == null || tcpPort.toInt() <= 0) {
            Toast.show("TCP端口必须为正整数且小于65536")
            return
        }
        if(udpPort.toShortOrNull() == null || udpPort.toInt() <= 0) {
            Toast.show("UDP端口必须为正整数且小于65536")
            return
        }

        val spEdit = sharedPreferences.edit()
        spEdit.putString("host",host)
        spEdit.putString("tcp_port",tcpPort)
        spEdit.putString("udp_port",udpPort)
        spEdit.putString("key",key)

        spEdit.putBoolean("is_opened",true)

        //标记是否已经配置过了，如果没有配置过就直接打开输入界面，配置过了就直接打开主页
        spEdit.putBoolean("is_inputted",true)
        spEdit.apply()
        this.toHome()
    }

    private fun toHome(){
        val intent = Intent(this, HomeActivity::class.java)
        this.startActivity(intent)
        this.finish()
    }
}