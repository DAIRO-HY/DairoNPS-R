use crate::dao::{channel_dao, client_dao};
use crate::application;
use hyperoute_macro::{GET, PUT};

/// 获取全局数据API
#[GET("/api/common/global")]
pub async fn global() -> model::GlobalData {
    model::GlobalData {
        is_need_restart: application::IS_NEED_RESTART.load(std::sync::atomic::Ordering::Acquire),
        is_restarting: application::IS_RESTARTING.load(std::sync::atomic::Ordering::Acquire),
    }
}

/// 获取全局数据API
#[PUT("/api/common/restart")]
pub async fn restart() {
    application::restart().await;
}

/// 这个模块定义了一个公共的控制器函数 `dropdown`，它根据路径参数 `tag` 的值来查询数据库中的不同表（如 `client` 或 `channel`），并返回一个包含标签和值的下拉列表数据结构。
#[GET("/api/common/dropdown/{tag}")]
pub async fn dropdown(tag: String) -> Vec<model::Dropdown> {
    let list = match tag.as_str() {
        "client" => client_dao::select_all(&lib_db::get()).await
            .unwrap_or_default()
            .into_iter()
            .map(|item| model::Dropdown {
                label: item.name,
                value: item.id.to_string(),
            })
            .collect::<Vec<_>>(),
        "channel" => channel_dao::select_all(&lib_db::get()).await
            .unwrap_or_default()
            .into_iter()
            .map(|item| model::Dropdown {
                label: item.name,
                value: item.id.to_string(),
            })
            .collect::<Vec<_>>(),
        _ => Vec::default(),
    };
    list
}

pub mod model {
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GlobalData {

        // 是否需要重启
        pub is_need_restart: bool,

        // 是否正在重启
        pub is_restarting: bool,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Dropdown {
        pub label: String,
        pub value: String,
    }
}
