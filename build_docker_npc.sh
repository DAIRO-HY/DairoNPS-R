#!/bin/bash

# 获取版本号
#version=$(grep -oP '(?<=version = ")[^"]+' ./lib_npc/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./lib_npc/Cargo.toml)

mkdir ./build/amd64
mkdir ./build/arm64
cp ./build/DairoNPC-linux-amd64-$version ./build/amd64/DairoNPC
cp ./build/DairoNPC-linux-arm64-$version ./build/arm64/DairoNPC

#github登录票据
github_token=$GITHUB_TOKEN

#docker用户名
docker_user=$DOCKER_USER
docker_pwd=$DOCKER_PASSWORD

#项目名
projectName="DairoNPC"


#---------------------------------------上传Docker镜像-----------------------------------------
echo "正在打包Docker镜像..."
cp ./document/docker-npc-build/Dockerfile .

docker login -u $docker_user --password $docker_pwd

#前提条件
#创建新的 builder
 #docker buildx create --name mybuilder --driver docker-container --use
 #docker buildx inspect --bootstrap
docker buildx build --platform linux/amd64,linux/arm64 -t $docker_user/dairo-npc:$version --push .
#docker push $docker_user/dairo-npc:$version
docker logout

rm Dockerfile

echo "---------------------------------------docker镜像推送完成--------------------------------------"
