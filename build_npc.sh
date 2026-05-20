#!/bin/bash

#打包amd64位linux二进制文件
docker run --platform linux/amd64 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"

mv ./target/release/DairoNPC ./DairoNPC-linux-amd64


#打包amd32位linux二进制文件
docker run --platform linux/386 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"

mv ./target/release/DairoNPC ./DairoNPC-linux-386


#打包arm64位linux二进制文件
docker run --platform linux/arm64 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"

mv ./target/release/DairoNPC ./DairoNPC-linux-arm64


#打包arm32位linux二进制文件
docker run --platform linux/arm/v7 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo --version && cargo clean && cargo build --release --package DairoNPC"

mv ./target/release/DairoNPC ./DairoNPC-linux-arm32-v7


#打包Android
cd ./DairoNPC-Android
./gradlew clean
./gradlew :app:assembleRelease
cd ../
mv ./DairoNPC-Android/app/build/outputs/apk/release/app-release.apk ./DairoNPC.apk


#编译到macOS intel平台
cargo clean
cargo build --release --package DairoNPC --target x86_64-apple-darwin
mv ./target/x86_64-apple-darwin/release/DairoNPC ./DairoNPC-mac-darwin-amd64


#编译到macOS M平台
cargo clean
cargo build --release --package DairoNPC --target aarch64-apple-darwin
mv ./target/aarch64-apple-darwin/release/DairoNPC ./DairoNPC-mac-darwin-aarch64