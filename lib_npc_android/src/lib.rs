use jni::objects::{JClass, JObject, JString};
use jni::sys::jshort;
use jni::{EnvUnowned, JValue, jni_sig, jni_str};
use lib_npc::application::Argument;
use std::sync::atomic::Ordering;
// #[unsafe(no_mangle)]
// pub extern "C" fn npc_start(host_p: *const c_char, tcp_port:i32, udp_port:i32, key_p: *const c_char) {
//     let (host,key) = unsafe{
//         (CStr::from_ptr(host_p).to_string_lossy().into_owned(),CStr::from_ptr(key_p).to_string_lossy().into_owned())
//     };
//     let args = Argument{
//         host,
//         tcp_port: tcp_port as u16,
//         udp_port: udp_port as u16,
//         key
//     };
//     std::thread::spawn(|| {
//         lib_npc::start(Some(args));
//     });
// }
//
// #[unsafe(no_mangle)]
// pub extern "C" fn npc_stop() {
//     lib_npc::stop();
// }
//
// #[unsafe(no_mangle)]
// pub extern "C" fn npc_bridge_count()-> i32 {
//     let count = lib_npc::application::BRIDGE_COUNT.load(Ordering::Relaxed);
//     count as i32
// }

/// 启动NPC服务
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_NPCRustBridge_open<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JClass<'local>,
    host: JString<'local>,
    tcp_port: jshort,
    udp_port: jshort,
    key: JString<'local>,
) {
    unowned_env
        .with_env(|_| -> jni::errors::Result<_> {
            //JString的值必须要在这里才能被读取出来，因为一旦这个函数返回，JNI的局部引用就会被释放掉，JString对象也就无法再被访问了。
            let host = host.to_string();
            let tcp_port = tcp_port.cast_unsigned();
            let udp_port = udp_port.cast_unsigned();
            let key = key.to_string();
            std::thread::spawn(move || {
                lib_npc::start(Some(Argument {
                    host,
                    tcp_port,
                    udp_port,
                    key,
                }));
            });
            Ok(())
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// 停止NPC服务
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_NPCRustBridge_close() {
    lib_npc::stop();
}

/// 获取NPC信息
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_NPCRustBridge_getInfo<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
) -> JObject<'local> {
    // 使用 with_env 获取可用的 Env 引用
    let rs = unowned_env.with_env(|env| -> jni::errors::Result<_> {
        let class = jni_str!("cn/dairo/npc/bean/NpcInfo");
        let ctor_id = env.get_method_id(
            class,
            jni_str!("<init>"),
            &jni_sig!((clientId: jlong, version: java.lang.String) -> void),
        )?;

        // 获取客户端ID
        let client_id = lib_npc::application::CLIENT_ID.load(Ordering::Relaxed);

        // NPC版本号
        let version = JString::from_str(env, lib_npc::application::VERSION)?;

        // 创建对象
        let obj = unsafe {
            env.new_object_unchecked(
                &class,
                ctor_id,
                &[
                    JValue::Long(client_id).as_jni(),
                    JValue::from(&version).as_jni(),
                ],
            )?
        };
        Ok(obj)
    });
    rs.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// 获取NPC运行状态
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_NPCRustBridge_getStatus<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
) -> JObject<'local> {
    // 使用 with_env 获取可用的 Env 引用
    let rs = unowned_env.with_env(|env| -> jni::errors::Result<_> {
        let class = jni_str!("cn/dairo/npc/bean/NpcStatus");
        let ctor_id = env
            // .get_method_id(class, jni_str!("<init>"), &jni_sig!("()V"))?;
            .get_method_id(
                class,
                jni_str!("<init>"),
                &jni_sig!(
                    (
                        isOpened: jboolean,
                        isRunning: jboolean,
                        connectMsg: java.lang.String,
                        bridgeCount: jint,
                        poolCount: jint,
                        inLen: jlong,
                        outLen: jlong
                    ) -> void),
            )?;

        // 获取桥接数量
        let bridge_count = lib_npc::application::BRIDGE_COUNT.load(Ordering::Relaxed) as i32;

        // 获取线程池数量
        let pool_count = lib_npc::application::POOL_COUNT.load(Ordering::Relaxed) as i32;

        let connect_msg =
            JString::from_str(env, &*lib_npc::application::NPC_CONNECT_MSG.lock().unwrap())
                .unwrap();
        let args = [
            JValue::Bool(*lib_npc::application::IS_OPENED.lock().unwrap()).as_jni(),
            JValue::Bool(*lib_npc::application::IS_NPC_RUNNING.lock().unwrap()).as_jni(),
            JValue::from(&connect_msg).as_jni(),
            JValue::Int(bridge_count).as_jni(),
            JValue::Int(pool_count).as_jni(),
            JValue::Long(lib_npc::application::IN_LEN.load(Ordering::Relaxed) as i64).as_jni(),
            JValue::Long(lib_npc::application::OUT_LEN.load(Ordering::Relaxed) as i64).as_jni(),
        ];

        // 创建对象
        let obj = unsafe {
            let obj = env.new_object_unchecked(&class, ctor_id, &args)?;

            // // 设置字段值
            // let field_id = env.get_field_id(jni_str!("cn/dairo/npc/User"), jni_str!("age"), jni_sig!("I"))?;
            // env.set_field_unchecked(&obj, field_id, JValue::Int(15))?;

            obj
        };
        Ok(obj)
    });
    rs.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}
