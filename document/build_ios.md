# 编译 Rust 为静态库

## Intel 模拟器
```bash
cargo build --release --package lib_npc_ios --target x86_64-apple-ios
```

## Apple Silicon 模拟器
```bash
cargo build --release --package lib_npc_ios --target aarch64-apple-ios-sim
```

## 真机
```bash
cargo build --release --package lib_npc_ios --target aarch64-apple-ios
```

#cbindgen ./lib_npc_ios --output ./target/lib_npc_ios.h
