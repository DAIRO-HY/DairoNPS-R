package cn.dairo.npc.util.share_preferences


/// <summary>
/// 会员信息存储
/// </summary>
object UserShared {
    private val sp: SpBase
        get() {
            return SpBase("common")
        }

    /**
     * 用户登录票据
     */
    var token: String?
        get() {
            return sp.getString("TOKEN")
        }
        set(value) {
            sp.put("TOKEN", value)
        }
    /**
     * 用户登录名
     */
    var userName: String
        get() {
            return sp.getString("USER_NAME") ?: ""
        }
        set(value) {
            sp.put("USER_NAME", value)
        }

    /// <summary>
    /// 是否登录
    /// </summary>
    val isLogin: Boolean
        get() = token != null

    /// <summary>
    /// 退出登录
    /// </summary>
    fun logout() {
        sp.clear()
    }
}
