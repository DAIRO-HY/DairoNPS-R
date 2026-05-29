#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."

#开发工具安装目录
develop_tool_dir=$(echo ~)/develop_tool

# 获取版本号
#version=$(grep -oP '(?<=version = ")[^"]+' ./lib_npc/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./lib_npc/Cargo.toml)

#打包amd64位linux二进制文件
docker run --platform linux/amd64 -it --rm -v ./:/home/rust/src -v $develop_tool_dir/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"
#mv ./target/release/DairoNPC ./build/DairoNPC-linux-amd64-$version

#
##打包amd32位linux二进制文件
#docker run --platform linux/386 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
#sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"
#mv ./target/release/DairoNPC ./build/DairoNPC-linux-386-$version
#
#
##打包arm64位linux二进制文件
#docker run --platform linux/arm64 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
#sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"
#mv ./target/release/DairoNPC ./build/DairoNPC-linux-arm64-$version
#
#
##打包arm32位linux二进制文件
#docker run --platform linux/arm/v7 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
#sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"
#mv ./target/release/DairoNPC ./build/DairoNPC-linux-arm32-v7-$version
