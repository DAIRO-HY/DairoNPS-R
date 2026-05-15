use lib_npc::application::Argument;
use std::ffi::{CStr, CString, c_char};
use std::sync::atomic::Ordering;

///NPC信息
#[repr(C)]
pub struct NpcInfo {
    ///客户端ID
    pub client_id: i64,
    ///NPC版本号
    pub version: *const c_char,
}

/**
 * Npc状态信息
 */
#[repr(C)]
pub struct NpcStatus {
    /// NPC打开状态
    pub is_opened: bool,

    /// NPC正在运行
    pub is_running: bool,

    /// npc连接消息
    pub connect_msg: *const c_char,

    /// 当前桥接数量
    pub bridge_count: u16,

    /// 当前连接池数量
    pub pool_count: u16,

    /// 入网流量
    pub in_len: u64,

    /// 出网流量
    pub out_len: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn npc_open(
    host_p: *const c_char,
    tcp_port: i32,
    udp_port: i32,
    key_p: *const c_char,
) {
    let (host, key) = unsafe {
        (
            CStr::from_ptr(host_p).to_string_lossy().into_owned(),
            CStr::from_ptr(key_p).to_string_lossy().into_owned(),
        )
    };
    let args = Argument {
        host,
        tcp_port: tcp_port as u16,
        udp_port: udp_port as u16,
        key,
    };
    std::thread::spawn(|| {
        lib_npc::start(Some(args));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn npc_close() {
    lib_npc::stop();
}

/// 获取NPC信息
#[unsafe(no_mangle)]
pub extern "C" fn npc_get_info() -> *const NpcInfo {
    let user = NpcInfo {
        client_id: 10010,
        version: CString::new("Tom").unwrap().into_raw(),
    };
    Box::into_raw(Box::new(user))
}

/// 释放NPC信息内存
#[unsafe(no_mangle)]
pub extern "C" fn npc_free_info(ptr: *const NpcInfo) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let user = Box::from_raw(ptr as *mut NpcInfo);

        if !user.version.is_null() {
            let _ = CString::from_raw(user.version as *mut c_char);
        }

        // user 离开作用域后
        // Box 自动释放结构体本身
    }
}

/// 获取NPC实时状态
#[unsafe(no_mangle)]
pub extern "C" fn npc_get_status() -> *const NpcStatus {
    let status = NpcStatus {
        is_opened: *lib_npc::application::IS_OPENED.lock().unwrap(),
        is_running: *lib_npc::application::IS_NPC_RUNNING.lock().unwrap(),
        connect_msg: CString::new(
            lib_npc::application::NPC_CONNECT_MSG
                .lock()
                .unwrap()
                .as_str(),
        )
            .unwrap()
            .into_raw(),
        bridge_count: lib_npc::application::BRIDGE_COUNT.load(Ordering::Relaxed),
        pool_count: lib_npc::application::POOL_COUNT.load(Ordering::Relaxed),
        in_len: lib_npc::application::IN_LEN.load(Ordering::Relaxed),
        out_len: lib_npc::application::OUT_LEN.load(Ordering::Relaxed),
    };
    Box::into_raw(Box::new(status))
}

/// 释放NPC实时状态内存
#[unsafe(no_mangle)]
pub extern "C" fn npc_free_status(ptr: *const NpcStatus) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let status = Box::from_raw(ptr as *mut NpcStatus);

        if !status.connect_msg.is_null() {
            let _ = CString::from_raw(status.connect_msg as *mut c_char);
        }
        // Box 自动释放结构体本身
    }
}
