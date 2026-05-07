# 编译 Rust 为静态库

## Intel 模拟器
```bash
cargo build --release --package npc_lib_ios --target x86_64-apple-ios
```

## Apple Silicon 模拟器
```bash
cargo build --release --package npc_lib_ios --target aarch64-apple-ios-sim
```

## 真机
```bash
cargo build --release --package npc_lib_ios --target aarch64-apple-ios
```

#cbindgen ./npc_lib_ios --output ./target/npc_lib_ios.h
