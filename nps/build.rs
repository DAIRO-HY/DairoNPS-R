use source_gen::*;

/// 追求极致的性能，编译时生成静态资源路由代码块，避免运行时扫描文件系统和猜测 MIME 类型的开销
/// 但是付出的代价是：
/// 1. 需要重新编译才能更新静态资源（适合版本化资源，如带 hash 的文件）
/// 2. 生成的代码可能很大，增加编译时间和二进出文件大小（但运行时性能提升明显）
fn main() {

    // 生成静态资源路由的代码块
    make_resource_route::make("assets/resources", 86400);

    // 生成 DAO 相关的代码块
    source_gen::make_dao::make("assets/sql", "assets/mapper");

    // 追踪输入文件变化
    println!("cargo:rerun-if-changed=assets/resources/");
    println!("cargo:rerun-if-changed=assets/sql/");
    println!("cargo:rerun-if-changed=assets/mapper/");
}
