#!/bin/bash

#github登录票据
github_token=$GITHUB_TOKEN

#docker用户名
docker_user=$DOCKER_USER
docker_pwd=$DOCKER_PWD

#项目名
projectName="DairoNPS-R"

repo="DAIRO-HY/$projectName"
branch="main"

#最终编译好的二进制文件
exec_name=dairo-nps-linux-amd64
exec_file="./$exec_name"


#--------------------------------------编译Linux二进制文件-----------------------------------------START
cargo --version && cargo clean && cargo build --release --package DairoNPC
#--------------------------------------编译Linux二进制文件-----------------------------------------END


echo "---------------------------------------docker镜像推送完成--------------------------------------"
ls
pwd
echo "---------------------------------------docker镜像推送完成--------------------------------------"
