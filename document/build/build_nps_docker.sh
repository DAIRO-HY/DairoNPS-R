#!/bin/bash

# 进入项目根目录
cd "$(dirname "$0")/../.."

# 获取版本号
#version=$(grep -oP '(?<=version = ")[^"]+' ./DairoNPS/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' ./DairoNPS/Cargo.toml)

mkdir ./build/amd64
mkdir ./build/arm64
cp ./build/DairoNPS-linux-amd64-$version ./build/amd64/DairoNPS
cp ./build/DairoNPS-linux-arm64-$version ./build/arm64/DairoNPS

#github登录票据
github_token=$GITHUB_TOKEN

#docker用户名
docker_user=$DOCKER_USER
docker_pwd=$DOCKER_PASSWORD

#项目名
projectName="DairoNPS"


#---------------------------------------上传Docker镜像-----------------------------------------
echo "正在打包Docker镜像..."
cp ./document/build/nps_dockerfile/Dockerfile ./

docker login -u $docker_user --password $docker_pwd

#前提条件
#创建新的 builder
 docker buildx create --name mybuilder --driver docker-container --use
 docker buildx inspect --bootstrap
docker buildx build --platform linux/amd64,linux/arm64 -t $docker_user/dairo-nps:$version --push .
#docker push $docker_user/dairo-nps:$version
docker logout

rm Dockerfile

echo "---------------------------------------docker镜像推送完成--------------------------------------"
