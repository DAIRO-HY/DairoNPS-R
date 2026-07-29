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

mkdir /home/rust/src
cd /home/rust/src

#--------------------------------------获取代码-----------------------------------------
if [ -d $projectName ]; then
    echo "项目已存在，正在更新代码..."
    cd $projectName

    #删除所有新添加的文件
    git clean -f

    #取消所有更改
    git reset --hard
    git pull
else
    CLONE_URL="https://${github_token}@github.com/${repo}.git"
    echo "正在克隆代码，URL:$CLONE_URL 分支：$branch..."
    git clone --branch $branch $CLONE_URL
    cd $projectName
fi

#--------------------------------------编译Linux二进制文件-----------------------------------------START
#cargo --version && cargo clean && cargo build --release --package DairoNPC
#--------------------------------------编译Linux二进制文件-----------------------------------------END


#####################################################################安装JDK-START#######################################

## 将jdk压缩包添加到容器的 /root 目录,ADD指令会自动解压,解压之后的文件名:jdk1.8.0_241
## openjdk官方下载地址https://jdk.java.net/archive/
if [ ! -d /usr/local/jdk/jdk-17.0.1 ]; then
    cd /tmp
    curl -o openjdk.tar.gz https://download.java.net/java/GA/jdk17.0.1/2a2082e5a09d4267845be086888add4f/12/GPL/openjdk-17.0.1_linux-x64_bin.tar.gz
    tar -zvxf openjdk.tar.gz
    rm -rf openjdk.tar.gz
    mv jdk-17.0.1 /usr/local/jdk/jdk-17.0.1
else
    echo "JDK already exists."
fi


# 配置JAVA_HOME环境变量
export JAVA_HOME="/usr/local/jdk/jdk-17.0.1/"

# 将JAVA_HOME/bin 添加至PATH环境变量
export PATH=$JAVA_HOME/bin:$PATH
java --version
####################################################################安装JDK-END##########################################

#--------------------------------------编译Android-----------------------------------------START
if [ ! -d /usr/local/AndroidSDK/cmdline-tools/latest ]; then
    mkdir -p /usr/local/AndroidSDK/cmdline-tools
    cd /usr/local/AndroidSDK/cmdline-tools
    curl -o commandlinetools.zip https://dl.google.com/android/repository/commandlinetools-linux-14742923_latest.zip
    unzip commandlinetools.zip
    rm commandlinetools.zip
    mv cmdline-tools latest
else
  echo "Android SDK command line tools already exist."
fi
ANDROID_HOME="/usr/local/AndroidSDK"
export PATH="$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools:$ANDROID_HOME/tools/bin:$ANDROID_HOME/cmdline-tools/latest/bin:${PATH}"

#安装NDK
yes|sdkmanager "ndk;30.0.14904198"

#安装build-tools
yes|sdkmanager "build-tools;36.0.0"

#安装platform-tools
yes|sdkmanager "platform-tools"

#安装platforms
yes|sdkmanager "platforms;android-36"

#添加授权
yes|sdkmanager --licenses

#set -e


#rustup target add aarch64-linux-android
#rustup target add armv7-linux-androideabi
#rustup target add x86_64-linux-android
#cargo install cargo-ndk

cd "/home/rust/src/$projectName"

# 编译目标平台
cargo ndk build --release  --package lib_npc_android --target arm64-v8a
cargo ndk build --release  --package lib_npc_android --target x86_64

mv ./target/aarch64-linux-android/release/liblib_npc_android.so ./DairoNPC-Android/app/src/main/jniLibs/arm64-v8a/libnpc_android.so
mv ./target/x86_64-linux-android/release/liblib_npc_android.so ./DairoNPC-Android/app/src/main/jniLibs/x86_64/libnpc_android.so

cd DairoNPC-Android
./gradlew :app:assembleRelease
#--------------------------------------编译Android-----------------------------------------END


echo "---------------------------------------docker镜像推送完成--------------------------------------"
ls
pwd
echo "---------------------------------------docker镜像推送完成--------------------------------------"
