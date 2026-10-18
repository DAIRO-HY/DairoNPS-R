use std::collections::HashMap;
use hyperoute_codegen::{Embed, ParamNaming, StaticDir};
use std::env;
use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() {

    // 生成 DAO 相关的代码块
    generate_dao().await;
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_dir = manifest_dir.join("src");

    // 扫描controller、filter下的注解，并挂载静态资源，生成路由代码
    let routes = hyperoute_codegen::Scanner::new()
        .dir(src_dir.join("web/controller"), "crate::web::controller")
        .dir(src_dir.join("web/filter"), "crate::web::filter")
        // 普通参数同时接收驼峰（clientId）和Rust原名（client_id）
        .param_naming(ParamNaming::CamelCase)
        .static_dir(
            StaticDir::new("/", manifest_dir.join("assets/resources"))
                // 始终打包进二进制，发布时只有一个文件
                .embed(Embed::Always)
                // html里的 {{template include/head}} 先合并成完整页面
                .html_include(true)
                // 浏览器每次都带ETag来验证，没变就是304，部署后能立即拿到新资源
                .cache_control("no-cache")
                .gzip(true),
        )
        .scan();
    let code = hyperoute_codegen::generate_router_source(&routes);
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("routes.rs");
    fs::write(&out_path, code).unwrap();

    // 追踪输入文件变化
    println!("cargo:rerun-if-changed=src/web/controller");
    println!("cargo:rerun-if-changed=src/web/filter");
    println!("cargo:rerun-if-changed=assets/resources/");

}

// 生成 DAO 相关的代码块
async fn generate_dao() {

    // -------------------------------------Dao自动生成------------------START

    //初始化数据库,这个数据用来支持sqlx宏编译检查sql语句的正确性，必须在编译阶段就初始化数据库，否则sqlx会报错
    lib_db::init("../data/dairo-nps.sqlite").await;

    // 字段名 -> Rust类型 的覆盖规则(本项目自定义的字段命名约定)
    let type_overrides = HashMap::from([]);

    // 生成 DAO 相关的代码块
    sqlx_dsl_dao::generate(lib_db::get(), "./assets/dsl", &type_overrides).await;

    // -------------------------------------Dao自动生成------------------END

    println!("cargo:rerun-if-changed=assets/ddl/");
    println!("cargo:rerun-if-changed=assets/dsl/");
    println!("cargo:rerun-if-changed=../data/dairo-nps.sqlite");
}

