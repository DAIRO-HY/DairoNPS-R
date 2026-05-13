use jni::objects::{JClass, JObject, JString};
use jni::sys::jshort;
use jni::{jni_sig, jni_str, EnvUnowned, JValue};
use npc_lib::application::Argument;
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
//         npc_lib::start(Some(args));
//     });
// }
//
// #[unsafe(no_mangle)]
// pub extern "C" fn npc_stop() {
//     npc_lib::stop();
// }
//
// #[unsafe(no_mangle)]
// pub extern "C" fn npc_bridge_count()-> i32 {
//     let count = npc_lib::application::BRIDGE_COUNT.load(Ordering::Relaxed);
//     count as i32
// }

/// 启动NPC服务
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_RustBridge_start<'local>(
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
            npc_lib::start(Some(Argument {
                host: host.to_string(),
                tcp_port: tcp_port.cast_unsigned(),
                udp_port: udp_port.cast_unsigned(),
                key: key.to_string(),
            }));
            Ok(())
        })
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// 停止NPC服务
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_RustBridge_stop() {
    npc_lib::stop();
}

// This `#[no_mangle]` keeps rust from "mangling" the name and making it unique
// for this crate. The name follows a strict naming convention so that the JNI
// implementation will be able to automatically find the implementation of a
// native method based on its name.
//
// The `'local` lifetime here represents the JNI local reference frame that was
// setup by the JVM before calling this native method. By explicitly
// naming this lifetime it's possible to associate new local references with the
// same lifetime and return those to the caller.
//
// Note that, giving JNI stack frames a lifetime name and explicitly tracking
// thread attachments are important safety features for `jni-rs`.
//
// Safety:
//
// This is only safe if the signature matches the ABI that the JVM expects
//
// The lifetime of the caller frame must not be declared as `'static`
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_RustBridge_getHello<'local>(
    // This `unowned_env` represents the fact that the JVM has implicitly
    // attached the current thread to the JVM (so you don't need to call
    // `JavaVM::attach_current_thread` before using JNI)
    //
    // Always use `EnvUnowned` to capture raw `jni::sys::JNIEnv` pointers passed
    // to native methods, so that you can associate the pointer with a JNI stack
    // frame lifetime and safely use `EnvUnowned::with_env`.
    mut unowned_env: EnvUnowned<'local>,
    // This is the class that owns our static method. Not going to be used, but
    // still needs to have an argument slot.
    // If this were a non-static method, this argument would be `this: JObject<'local>`
    _class: JClass<'local>,
    input: JString<'local>,
) -> JString<'local> {
    // Before we can start using the [`Env`] API we need to tell `jni-rs`
    // about the "unowned" thread attachment and map the raw pointer into a
    // non-transparent [`Env`] that is (internally) associated with a thread
    // attachment guard.
    let outcome = unowned_env.with_env(|env| -> jni::errors::Result<_> {
        let input: String = input.to_string();

        let test_info = "";

        env.new_string(format!("Hello, {}!{}", input, test_info))
    });

    // Finally, we have to resolve the `Outcome` into a concrete return value.
    //
    // Our code above may have failed with a JNI error, or some other
    // application-specific error, or it may have panicked. None of these things
    // can pass over the FFI boundary back into the JVM.
    //
    // Mapping of errors and panics is done according to the selected
    // `ErrorPolicy` trait implementation which is able to use JNI for throwing
    // Java exceptions if necessary.
    //
    // This design lets you encapsulate your own approach to forwarding errors
    // and panics to Java code and then easily reuse it across multiple native
    // methods.
    //
    // In this case we use a built-in policy that throws a Java
    // `RuntimeException` with a message containing the error/panic details.
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// 获取NPC信息
#[unsafe(no_mangle)]
pub extern "system" fn Java_cn_dairo_npc_RustBridge_getInfo<'local>(
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
        let client_id = npc_lib::application::CLIENT_ID.load(Ordering::Relaxed);

        // NPC版本号
        let version = JString::from_str(env, env!("CARGO_PKG_VERSION"))?;

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
pub extern "system" fn Java_cn_dairo_npc_RustBridge_getStatus<'local>(
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
                &jni_sig!((isOpened: jboolean, isRunning: jboolean, connectMsg: java.lang.String, bridgeCount: jint, poolCount: jint) -> void),
            )?;

        // 获取桥接数量
        let bridge_count = npc_lib::application::BRIDGE_COUNT.load(Ordering::Relaxed) as i32;

        // 获取线程池数量
        let pool_count = npc_lib::application::POOL_COUNT.load(Ordering::Relaxed) as i32;

        let connect_msg = JString::from_str(env, &*npc_lib::application::NPC_CONNECT_MSG.lock().unwrap()).unwrap();
        let args = [
            JValue::Bool(*npc_lib::application::IS_OPENED.lock().unwrap()).as_jni(),
            JValue::Bool(*npc_lib::application::IS_NPC_RUNNING.lock().unwrap()).as_jni(),
            JValue::from(&connect_msg).as_jni(),
            JValue::Int(bridge_count).as_jni(),
            JValue::Int(pool_count).as_jni(),
        ];


        // 创建对象
        let obj = unsafe {
            let obj = env.new_object_unchecked(
                &class,
                ctor_id,
                &args,
            )?;

            // // 设置字段值
            // let field_id = env.get_field_id(jni_str!("cn/dairo/npc/User"), jni_str!("age"), jni_sig!("I"))?;
            // env.set_field_unchecked(&obj, field_id, JValue::Int(15))?;

            obj
        };
        Ok(obj)
    });
    rs.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}
