#!/bin/bash

#打包amd64位linux二进制文件
docker run --platform linux/amd64 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"

mv ./target/release/DairoNPS ./DairoNPS-linux-amd64


#打包amd32位linux二进制文件
docker run --platform linux/386 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"

mv ./target/release/DairoNPS ./DairoNPS-linux-386


#打包arm64位linux二进制文件
docker run --platform linux/arm64 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"

mv ./target/release/DairoNPS ./DairoNPS-linux-arm64


#打包arm32位linux二进制文件
docker run --platform linux/arm/v7 -it --rm -v ./:/home/rust/src -v $HOME/.cargo:/root/.cargo -w /home/rust/src rust:1.93 \
sh -c "cargo clean && cargo build --release --package DairoNPS"

mv ./target/release/DairoNPS ./DairoNPS-linux-arm32-v7


#编译到macOS intel平台
cargo clean
cargo build --release --package DairoNPS --target x86_64-apple-darwin
mv ./target/x86_64-apple-darwin/release/DairoNPS ./DairoNPS-mac-darwin-amd64


#编译到macOS M平台
cargo clean
cargo build --release --package DairoNPS --target aarch64-apple-darwin
mv ./target/aarch64-apple-darwin/release/DairoNPS ./DairoNPS-mac-darwin-aarch64