#!/bin/bash

# 当前 sh 文件所在目录
script_dir="$(cd "$(dirname "$0")" && pwd)"

# 项目目录
project_root="$(cd "$script_dir/../.." && pwd)"

#开发工具安装目录
develop_tool_dir=$(echo ~)/develop_tool

# 获取版本号
version=$(sed -n 's/^version = "\(.*\)"/\1/p' $project_root/lib_npc/Cargo.toml)

#先创建文件夹
mkdir $project_root/build

#--------------------------------------安装JDK--------------------------------------START

## 将jdk压缩包添加到容器的 /root 目录,ADD指令会自动解压,解压之后的文件名:jdk1.8.0_241
## openjdk官方下载地址https://jdk.java.net/archive/
if [ ! -d $develop_tool_dir/jdk-17.0.1 ]; then
    cd /tmp
    curl -o openjdk.tar.gz https://download.java.net/java/GA/jdk17.0.1/2a2082e5a09d4267845be086888add4f/12/GPL/openjdk-17.0.1_linux-x64_bin.tar.gz
    tar -zvxf openjdk.tar.gz
    rm openjdk.tar.gz
    mv jdk-17.0.1 $develop_tool_dir/jdk-17.0.1
else
    echo "JDK already exists."
fi


# 配置JAVA_HOME环境变量
export JAVA_HOME="${develop_tool_dir}/jdk-17.0.1"

# 将JAVA_HOME/bin 添加至PATH环境变量,只针对本次有效，如果需要永久生效，需要将下面两行添加到~/.bashrc或者~/.profile中
export PATH=$JAVA_HOME/bin:$PATH
java --version
#--------------------------------------安装JDK--------------------------------------END

#--------------------------------------安装Gradle--------------------------------------START
if [ ! -d $develop_tool_dir/gradle-9.5.0 ]; then
    cd /tmp
    curl -L -o gradle-9.5.0-bin.zip https://services.gradle.org/distributions/gradle-9.5.0-bin.zip
    unzip gradle-9.5.0-bin.zip
    rm gradle-9.5.0-bin.zip
    mv gradle-9.5.0 $develop_tool_dir/gradle-9.5.0
else
    echo "GRADLE already exists."
fi


# 配置GRADLE_HOME环境变量
export GRADLE_HOME="${develop_tool_dir}/gradle-9.5.0"

# 将GRADLE_HOME/bin 添加至PATH环境变量,只针对本次有效，如果需要永久生效，需要将下面两行添加到~/.bashrc或者~/.profile中
export PATH=$GRADLE_HOME/bin:$PATH

#修改 .gradle 存放位置,默认在~/.gradle
export GRADLE_USER_HOME="${develop_tool_dir}/.gradle"
gradle --version
#--------------------------------------安装Gradle--------------------------------------END

#--------------------------------------编译Android-----------------------------------------START
if [ ! -d $develop_tool_dir/AndroidSDK/cmdline-tools/latest ]; then
    mkdir -p $develop_tool_dir/AndroidSDK/cmdline-tools
    cd $develop_tool_dir/AndroidSDK/cmdline-tools
    curl -o commandlinetools.zip https://dl.google.com/android/repository/commandlinetools-linux-14742923_latest.zip
    unzip commandlinetools.zip
    rm commandlinetools.zip
    mv cmdline-tools latest
else
  echo "Android SDK command line tools already exist."
fi
ANDROID_HOME="${develop_tool_dir}/AndroidSDK"

export ANDROID_SDK_ROOT=$ANDROID_HOME
export PATH="$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools:$ANDROID_HOME/tools/bin:$ANDROID_HOME/cmdline-tools/latest/bin:${PATH}"

#修改 .android 存放位置,默认在~/.android
export ANDROID_USER_HOME="${develop_tool_dir}/.android"

sdkmanager --version

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
#--------------------------------------编译Android-----------------------------------------END

#进入项目根目录
cd $project_root/DairoNPC-Android/app
gradle clean
gradle assembleRelease

mv $project_root/DairoNPC-Android/app/build/outputs/apk/release/app-release.apk $project_root/build/DairoNPC-$version.apk
