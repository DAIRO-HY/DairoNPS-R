#!/bin/bash

# 事前准备
# 安装 iOS 编译目标
# rustup target add aarch64-apple-ios
# rustup target add aarch64-apple-ios-sim
# rustup target add x86_64-apple-ios
# 说明：
# target	用途
# aarch64-apple-ios	真机
# aarch64-apple-ios-sim	Apple Silicon 模拟器
# x86_64-apple-ios	Intel 模拟器


set -e

# 删除旧依赖
rm -rf ./DairoNPC-IOS/DairoNPC/NpcLibIOS.xcframework

# 编译目标平台
cargo build --release --package npc_lib_ios --target x86_64-apple-ios
cargo build --release --package npc_lib_ios --target aarch64-apple-ios-sim
cargo build --release --package npc_lib_ios --target aarch64-apple-ios

# 合并模拟器架构(苹果要求)
lipo -create target/aarch64-apple-ios-sim/release/libnpc_lib_ios.a target/x86_64-apple-ios/release/libnpc_lib_ios.a -output libnpc_lib_ios.a

# 创建 xcframework
xcodebuild -create-xcframework -library target/aarch64-apple-ios/release/libnpc_lib_ios.a -headers ./npc_lib_ios/include \
-library libnpc_lib_ios.a -headers ./npc_lib_ios/include \
-output ./DairoNPC-IOS/DairoNPC/NpcLibIOS.xcframework

# 后续操作
#  xcode配置桥接头部文件,TARGETS -> Build Settings -> 搜索(bridging) -> Objective-C Bridging Header
#  设置桥接文件路径如:DairoNPC/Npc-Bridging-Header.h