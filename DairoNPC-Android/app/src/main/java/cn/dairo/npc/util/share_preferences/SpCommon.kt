package cn.dairo.npc.util.share_preferences

/**
 * 通用配置文件
 */
object SpCommon {
    private val sp: SpBase
        get() {
            return SpBase("common")
        }

    /**
     * 记录分享者ID,会员注册的时候携带这个ID
     */
    var shareUserId: Int
        get() {
            return sp.getInt("SHARE_USER_ID")
        }
        set(value) {
            sp.put("SHARE_USER_ID", value)
        }

    /**
     * 记录已经打开的版本号
     */
    var usedVersionCode: Int
        get() {
            return sp.getInt("VERSION_CODE")
        }
        set(value) {
            sp.put("VERSION_CODE", value)
        }


    /// <summary>
    /// 记录上次获取推介页面数据的时间
    /// </summary>
    var lastUsefulLoadTime: Long
        get() {
            return this.sp.getLong("LAST_USEFUL_LOAD_TIME")
        }
        set(newValue) {
            this.sp.put("LAST_USEFUL_LOAD_TIME", newValue)
        }

}
