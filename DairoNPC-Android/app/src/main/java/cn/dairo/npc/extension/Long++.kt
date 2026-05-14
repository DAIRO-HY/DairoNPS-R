package cn.dairo.npc.extension


/**
 * 将字节数格式化为人类可读的容量字符串
 */
val Long.readableSize: String
    get() {
        val kb = 1024.0
        val mb = kb * 1024
        val gb = mb * 1024
        val tb = gb * 1024

        return when {
            this >= tb -> "%.2fTB".format(this / tb)
            this >= gb -> "%.2fGB".format(this / gb)
            this >= mb -> "%.2fMB".format(this / mb)
            this >= kb -> "%.2fKB".format(this / kb)
            else -> "${this}B"
        }
    }