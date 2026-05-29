#!/bin/bash

# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$script_dir/../.." && pwd)"

# 先安装cargo
. $script_dir/install_cargo.sh

#先创建文件夹
mkdir $project_root/build

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' $project_root/lib_npc/Cargo.toml)

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
#mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-arm64-$version
#
#
##打包arm32位linux二进制文件
#cargo --version && cargo clean && cargo build --release --package DairoNPC
#mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-arm32-v7-$version
