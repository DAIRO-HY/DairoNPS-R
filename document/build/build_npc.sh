#!/bin/bash

# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$SCRIPT_DIR/../.." && pwd)"

#开发工具安装目录
develop_tool_dir=$(echo ~)/develop_tool

# 获取版本号
#version=$(grep -oP '(?<=version = ")[^"]+' ./lib_npc/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' $project_root/lib_npc/Cargo.toml)

#先创建build文件夹
mkdir $develop_tool_dir/build

apt update
yes|apt install build-essential

#--------------------------------------安装CARGO--------------------------------------START# 指定安装目录
export RUSTUP_HOME=$develop_tool_dir/.rustup
export CARGO_HOME=$develop_tool_dir/.cargo

# 加入PATH
export PATH=${CARGO_HOME}/bin:${PATH}
curl https://sh.rustup.rs -sSf | sh -s -- -y     --default-toolchain 1.93.0

cargo --version
#--------------------------------------安装CARGO--------------------------------------START

#打包amd64位linux二进制文件
cargo --version && cargo clean && cargo build --release --package DairoNPC
mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-amd64-$version


##打包amd32位linux二进制文件
#cargo --version && cargo clean && cargo build --release --package DairoNPC
#mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-386-$version
#
#
##打包arm64位linux二进制文件
#cargo --version && cargo clean && cargo build --release --package DairoNPC
#$project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-arm64-$version
#
#
##打包arm32位linux二进制文件
#cargo --version && cargo clean && cargo build --release --package DairoNPC
#$project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-arm32-v7-$version
