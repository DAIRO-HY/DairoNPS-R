use std::collections::HashMap;

#[async_std::main]
async fn main() {
    println!("cargo:rerun-if-changed=assets/sql/");
    println!("cargo:rerun-if-changed=assets/dsl/");
    println!("cargo:rerun-if-changed=../data/farming2.sqlite");

    // 是否在编译期对文本类静态资源（html/css/js等）做 gzip 压缩，开关，按需切换
    const COMPRESS_RESOURCES: bool = true;

    const STATIC_PATH: &'static str = "assets/resources";

    // 追踪 static 目录下的文件/子目录变化，只有真正发生变化时才会重新执行下面的 make()
    axum_static_embed::watch_dir(STATIC_PATH);

    // root_dir = "static"，max_age = 3600 秒，compress = true（对 html/css/js 等文本资源做编译期 gzip）
    axum_static_embed::make(STATIC_PATH, 3600, true);

    // -------------------------------------Dao自动生成------------------START

    //初始化数据库,这个数据用来支持sqlx宏编译检查sql语句的正确性，必须在编译阶段就初始化数据库，否则sqlx会报错
    lib_db::init("../data/dairo-nps.sqlite").await;

    // 字段名 -> Rust类型 的覆盖规则(本项目自定义的字段命名约定)
    let type_overrides = HashMap::from([
        ("birthDate".to_string(), "String".to_string()),
        ("issueDate".to_string(), "String".to_string()),
        (
            "apply1Date".to_string(),
            "chrono::NaiveDateTime".to_string(),
        ),
        (
            "apply2Date".to_string(),
            "chrono::NaiveDateTime".to_string(),
        ),
    ]);

    // 生成 DAO 相关的代码块
    sqlx_dsl_dao::generate(lib_db::get(), "./assets/dsl", &type_overrides).await;

    // -------------------------------------Dao自动生成------------------END
}
