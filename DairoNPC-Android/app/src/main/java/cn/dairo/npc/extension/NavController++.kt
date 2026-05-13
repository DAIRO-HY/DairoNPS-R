package cn.dairo.npc.extension

import androidx.navigation.NavController

/**
 * 关闭所有已经打开的路由并跳转到新的路由
 */
fun NavController.relaunch(route: String) {
    this.navigate(route) {
        popUpTo(0) {
            inclusive = true
        }
    }
}