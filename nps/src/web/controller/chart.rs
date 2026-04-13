use crate::dao::traffic_stats_dao;
use crate::extension::ResponseEmptyExt;
use crate::extension::number::{Div, ToDataSize, ToDateFormat};
use crate::model::data_io_len::{DataIOLen, ToU64};
use crate::web::extract::AppQuery;
use crate::{forward, nps};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use chrono::DateTime;
use futures::{Stream, TryFutureExt};
use itertools::Itertools;
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

///　实时获取当前流量大小
pub async fn real_len(
    AppQuery(param): AppQuery<model::ChartParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::iter(0..).then(move |i| {
        let param = param.clone();
        async move {
            if i > 0 {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            let data = get_real_len(param).await;
            Ok(Event::default().data(data))
        }
    });
    Sse::new(stream)
}

/// 获取当前流量数据
async fn get_real_len(param: model::ChartParam) -> String {
    let channel_live_map = nps::CHANNEL_LIVE_MAP.lock().await;
    let forward_live_map = forward::FORWARD_LIVE_MAP.lock().await;
    let data_len = if param.channel_id > 0 {
        //查看该隧道实时流量记录
        match channel_live_map.get(&param.channel_id) {
            Some(v) => v.data_len.load(),
            None => DataIOLen::default(),
        }
    } else if param.client_id > 0 {
        //查看该客户端实时流量记录
        channel_live_map
            .iter()
            .filter_map(|(_, v)| {
                if v.client_id != param.client_id {
                    return None;
                }
                Some(v.data_len.load())
            })
            .fold(DataIOLen::default(), |pre, it| pre + it)
    } else if param.forward_id > 0 {
        //查看该端口转发实时流量记录
        match forward_live_map.get(&param.forward_id) {
            Some(v) => v.data_len.load(),
            None => DataIOLen::default(),
        }
    } else {
        //查看内网穿透+端口转发实时流量记录
        let channel_total = channel_live_map
            .iter()
            .fold(DataIOLen::default(), |pre, (_, it)| {
                pre + it.data_len.load()
            });
        let forward_total = forward_live_map
            .iter()
            .fold(DataIOLen::default(), |pre, (_, it)| {
                pre + it.data_len.load()
            });
        channel_total + forward_total
    };
    format!("{}:{}", data_len.in_len, data_len.out_len)
}

///　获取指定时间段的流量数据
pub async fn data_len(AppQuery(param): AppQuery<model::DataLenQuery>) -> Response {
    //时间间隔
    let time_jg = param.end_time - param.start_time;

    //统计时间最小单位长度
    let label_format = if time_jg <= 60 {
        //小于1分钟，则时间最小单位到秒（yyyyMMddHHmmss）
        "%Y-%m-%d %H:%M:%S"
    } else if time_jg <= 60 * 60 {
        //小于1小时，则时间最小单位到分（yyyyMMddHHmm）
        "%Y-%m-%d %H:%M"
    } else if time_jg <= 24 * 60 * 60 {
        //小于1天，则时间最小单位到小时（yyyyMMddHH）
        "%Y-%m-%d %H"
    } else if time_jg <= 31 * 24 * 60 * 60 {
        //小于31天，则时间最小单位到天（yyyyMMdd）
        "%Y-%m-%d"
    } else if time_jg <= 366 * 24 * 60 * 60 {
        //小于1一年，则时间最小单位到月（yyyyMM）
        "%Y-%m"
    } else {
        "%Y"
    };

    let data_len_list = if param.client_id > 0 {
        traffic_stats_dao::select_io_len_by_client(
            &db::get(),
            param.client_id,
            param.start_time,
            param.end_time,
        )
        .await
        .unwrap()
    } else if param.channel_id > 0 {
        traffic_stats_dao::select_io_len_by_channel(
            &db::get(),
            param.channel_id,
            param.start_time,
            param.end_time,
        )
        .await
        .unwrap()
    }  else if param.forward_id > 0 {
        traffic_stats_dao::select_io_len_by_forward(
            &db::get(),
            param.forward_id,
            param.start_time,
            param.end_time,
        )
        .await
        .unwrap()
    } else {
        traffic_stats_dao::select_io_len(&db::get(), param.start_time, param.end_time)
            .await
            .unwrap()
    };

    let label2data_len = data_len_list
        .iter()
        .map(|it| {
            let label = DateTime::from_timestamp_secs(it.date)
                .unwrap()
                .with_timezone(&chrono::Local)
                .format(label_format)
                .to_string();
            let data_len = DataIOLen {
                in_len: it.in_len as u64,
                out_len: it.out_len as u64,
            };
            (label, data_len)
        })
        .into_grouping_map()
        .fold(DataIOLen::default(), |pre, label, it| pre + it);

    //记录该时间段的最大数据长度
    let max_data_size = label2data_len
        .values()
        .map(|it| it.in_len.max(it.out_len))
        .max()
        .unwrap_or_default();

    let (unit_size, unit) = if max_data_size > 1024 * 1024 * 1024 {
        (1024 * 1024 * 1024, "GB")
    } else if max_data_size > 1024 * 1024 {
        (1024 * 1024, "MB")
    } else if max_data_size > 1024 {
        (1024, "KB")
    } else {
        (1, "B")
    };

    let mut loop_time = DateTime::from_timestamp_secs(param.start_time)
        .unwrap()
        .with_timezone(&chrono::Local);
    let end_time = DateTime::from_timestamp_secs(param.end_time)
        .unwrap()
        .with_timezone(&chrono::Local);

    //报表标题列表
    let mut labels: Vec<String> = Default::default();

    //入网数据列表
    let mut in_lens: Vec<String> = Default::default();

    //出网数据列表
    let mut out_lens: Vec<String> = Default::default();

    //为每个时间点生成数据
    while loop_time <= end_time {
        let label = loop_time.format(label_format).to_string();

        if let Some(it) = label2data_len.get(&label) {
            in_lens.push((it.in_len as f64).div(unit_size as f64, 2));
            out_lens.push((it.out_len as f64).div(unit_size as f64, 2));
        } else {
            in_lens.push(Default::default());
            out_lens.push(Default::default());
        }
        labels.push(label);

        match label_format {
            //精确到秒
            "%Y-%m-%d %H:%M:%S" => loop_time = loop_time + chrono::Duration::seconds(1),
            //精确到分
            "%Y-%m-%d %H:%M" => loop_time = loop_time + chrono::Duration::minutes(1),
            //精确到小时
            "%Y-%m-%d %H" => loop_time = loop_time + chrono::Duration::hours(1),
            //精确到天
            "%Y-%m-%d" => {
                loop_time = loop_time + chrono::Days::new(1);
            }
            //精确到月
            "%Y-%m" => {
                loop_time = loop_time + chrono::Months::new(1);
            }
            //精确到年
            "%Y" => {
                loop_time = loop_time + chrono::Months::new(12);
            }
            _ => {}
        }
    }
    Response::json(model::DataLenInfo {
        labels,
        in_lens,
        out_lens,
        unit: unit.to_string(),
    })
}

mod model {
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Debug, Clone, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChartParam {
        pub client_id: i64,
        pub channel_id: i64,
        pub forward_id: i64,
    }

    #[derive(Debug, Clone, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DataLenQuery {
        //客户端id
        pub client_id: i64,

        //隧道id
        pub channel_id: i64,

        //端口转发id
        pub forward_id: i64,

        //入网流量
        pub start_time: i64,

        // 出网流量
        pub end_time: i64,
    }

    #[derive(Debug, Clone, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DataLenInfo {
        /**
         * 统计表标题列表
         */
        pub labels: Vec<String>,

        /**
         * 入网流量
         */
        pub in_lens: Vec<String>,

        /**
         * 出网流量
         */
        pub out_lens: Vec<String>,

        /**
         * 单位
         */
        pub unit: String,
    }
}
