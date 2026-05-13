package cn.dairo.npc.extension

import android.content.Context
import android.widget.Toast
import androidx.datastore.preferences.preferencesDataStore

val Context.npcDataStore by preferencesDataStore(
    name = "npc"
)

fun Context.toast(msg: String) {
    Toast.makeText(this, msg, Toast.LENGTH_SHORT).show()
}