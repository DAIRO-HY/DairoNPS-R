#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."

# 获取版本号
#version=$(grep -oP '(?<=version = ")[^"]+' ./lib_npc/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoNPS/Cargo.toml)

#打包amd64位linux二进制文件
docker run --platform linux/amd64 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"
mv ./target/release/DairoNPS ./build/DairoNPS-linux-amd64-$version


#打包amd32位linux二进制文件
docker run --platform linux/386 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"
mv ./target/release/DairoNPS ./build/DairoNPS-linux-386-$version


#打包arm64位linux二进制文件
docker run --platform linux/arm64 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"
mv ./target/release/DairoNPS ./build/DairoNPS-linux-arm64-$version


#打包arm32位linux二进制文件
docker run --platform linux/arm/v7 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"
mv ./target/release/DairoNPS ./build/DairoNPS-linux-arm32-v7-$version


#编译到macOS intel平台
cargo clean
cargo build --release --package DairoNPS --target x86_64-apple-darwin
mv ./target/x86_64-apple-darwin/release/DairoNPS ./build/DairoNPS-mac-darwin-amd64-$version


#编译到macOS M平台
cargo clean
cargo build --release --package DairoNPS --target aarch64-apple-darwin
mv ./target/aarch64-apple-darwin/release/DairoNPS ./build/DairoNPS-mac-darwin-aarch64-$version