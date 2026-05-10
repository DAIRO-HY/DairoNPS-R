package cn.dairo.npc.util

import android.widget.Toast
import cn.dairo.npc.ThisApplication

object Toast {
    fun show(msg:String){
            Toast.makeText(ThisApplication.Companion.app,msg, Toast.LENGTH_SHORT).show()
    }
}