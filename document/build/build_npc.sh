#!/bin/bash

#前提条件 安装 QEMU binfmt,目的:在amd64位linux系统上编译arm64位linux二进制文件
docker run --privileged --rm tonistiigi/binfmt --install all


# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$script_dir/../.." && pwd)"

# 先安装cargo
#. $script_dir/install_cargo.sh

#先创建文件夹
mkdir $project_root/build

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' $project_root/lib_npc/Cargo.toml)

echo "--------------------"
echo version
echo "--------------------"

echo $MASTER_SRC_DIR

docker_rust_build(){
  #这里的代码目录一定是宿主机上的绝对路径,不能是相对路径,否则会报错
  docker run --platform "$1" --rm -v /home/docker/jenkins+jdk17+docker/data/develop_tool/project/DairoNPS-R:/home/rust/src -v /home/docker/jenkins+jdk17+docker/data/develop_tool/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
  sh -c "cargo clean && cargo build --release --package DairoNPC"
}

docker run --platform linux/arm64 --rm -v /home/docker/jenkins+jdk17+docker/data/develop_tool/project/DairoNPS-R:/home/rust/src -v /home/docker/jenkins+jdk17+docker/data/develop_tool/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
  sh -c "cargo clean && cargo build --release --package DairoNPC"

#打包amd64位linux二进制文件
docker_rust_build linux/amd64
mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-amd64-$version


#打包amd32位linux二进制文件
docker_rust_build linux/386
mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-386-$version


#打包arm64位linux二进制文件
docker_rust_build linux/arm64
mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-arm64-$version


#打包arm32位linux二进制文件
docker_rust_build linux/arm/v7
mv $project_root/target/release/DairoNPC $project_root/build/DairoNPC-linux-arm32-v7-$version
