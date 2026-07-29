FROM rust:1.93

#RUN apt update

#安装curl
#RUN yes|apt install curl

# 安装 Android 编译目标
RUN rustup target add aarch64-linux-android
RUN rustup target add armv7-linux-androideabi
RUN rustup target add x86_64-linux-android
RUN cargo install cargo-ndk

WORKDIR /home/rust/src
#COPY build-linux.sh build-linux.sh
#RUN chmod +x build-linux.sh

#运行程序
ENTRYPOINT "./build-linux.sh"
