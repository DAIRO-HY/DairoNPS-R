#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."

# 获取版本号
#version=$(grep -oP '(?<=version = ")[^"]+' ./lib_npc/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoNPS/Cargo.toml)


#编译到macOS intel平台
cargo clean
cargo build --release --package DairoNPS --target x86_64-apple-darwin
mv ./target/x86_64-apple-darwin/release/DairoNPS ./build/DairoNPS-mac-darwin-amd64-$version


#编译到macOS M平台
cargo clean
cargo build --release --package DairoNPS --target aarch64-apple-darwin
mv ./target/aarch64-apple-darwin/release/DairoNPS ./build/DairoNPS-mac-darwin-aarch64-$version