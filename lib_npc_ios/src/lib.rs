use std::ffi::{c_char, CStr};
use std::sync::atomic::Ordering;
use lib_npc::application::Argument;

#[unsafe(no_mangle)]
pub extern "C" fn npc_start(host_p: *const c_char, tcp_port:i32, udp_port:i32, key_p: *const c_char) {
    let (host,key) = unsafe{
        (CStr::from_ptr(host_p).to_string_lossy().into_owned(),CStr::from_ptr(key_p).to_string_lossy().into_owned())
    };
    let args = Argument{
        host,
        tcp_port: tcp_port as u16,
        udp_port: udp_port as u16,
        key
    };
    std::thread::spawn(|| {
        lib_npc::start(Some(args));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn npc_stop() {
    lib_npc::stop();
}

#[unsafe(no_mangle)]
pub extern "C" fn npc_bridge_count()-> i32 {
    let count = lib_npc::application::BRIDGE_COUNT.load(Ordering::Relaxed);
    count as i32
}
