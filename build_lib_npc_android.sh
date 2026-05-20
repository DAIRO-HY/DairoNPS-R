#!/bin/bash

# 事前准备
# 安装 Android 编译目标
# rustup target add aarch64-linux-android
# rustup target add armv7-linux-androideabi
# rustup target add x86_64-linux-android

# cargo install cargo-ndk

set -e

# 编译目标平台
cargo ndk build --release  --package lib_npc_android --target arm64-v8a
cargo ndk build --release  --package lib_npc_android --target x86_64

mv ./target/aarch64-linux-android/release/liblib_npc_android.so ./DairoNPC-Android/app/src/main/jniLibs/arm64-v8a/libnpc_android.so
mv ./target/x86_64-linux-android/release/liblib_npc_android.so ./DairoNPC-Android/app/src/main/jniLibs/x86_64/libnpc_android.so

# 后续操作