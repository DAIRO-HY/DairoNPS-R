use crate::extension::number::ToDataSize;
use crate::extension::ResponseEmptyExt;
use crate::model::bytes_io::BytesIO;
use crate::nps;
use axum::extract::Query;
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use futures::{Stream, TryFutureExt};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;

///　获取流量数据
pub async fn get_bytes_io(
    Query(param): Query<model::ChartParam>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::iter(0..).then(move |_|{
        let param = param.clone();
        async move{
            tokio::time::sleep(Duration::from_secs(1)).await;
            let data = get_data(param).await;
            Ok(Event::default().data(data))
        }
    });
    Sse::new(stream)
}

/// 获取流量数据
async fn get_data(param: model::ChartParam) -> String {
    let channel_nps_map = nps::CHANNEL_NPS_MAP.lock().await;
    let bytes_io = if let Some(channel_id) = param.channel_id {
        match channel_nps_map.get(&channel_id) {
            Some(v) => v.data_total.load(),
            None => BytesIO::default(),
        }
    } else if let Some(client_id) = param.client_id {
        channel_nps_map
            .iter()
            .filter_map(|(_, v)| {
                if v.client_id != client_id {
                    return None;
                }
                Some(v.data_total.load())
            })
            .fold(BytesIO::default(), |pre, it| pre + it)
    } else {
        channel_nps_map
            .iter()
            .fold(BytesIO::default(), |pre, (_, it)| pre + it.data_total.load())
    };
    format!("{}:{}", bytes_io.in_bytes, bytes_io.out_bytes)
}

mod model {
    use serde::Deserialize;

    #[derive(Debug, Clone, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChartParam {
        pub client_id: Option<i64>,
        pub channel_id: Option<i64>,
        pub forward_id: Option<i64>,
    }
}
