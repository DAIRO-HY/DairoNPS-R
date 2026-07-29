use crate::util::server_image;
use axum::response::{IntoResponse, Response};
use lib_axum_extract::response::{AppError, AppResponse, ResponseExt};
use std::fs;
use axum::body::Body;
use axum::http::{header, StatusCode};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

/// 返回图片文件
pub async fn show(img: model::ShowImage) -> AppResponse {
    // 这里假设你已经从数据库查到了图片路径
    let thumb_path =
        server_image::thumb_path(&img.path, img.max_size, img.width, img.height).await?;
    if !thumb_path.exists() {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    let file = File::open(&thumb_path)
        .await
        .map_err(|_| AppError::OtherError("文件不存在"))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    Ok(([(header::CONTENT_TYPE, "image/jpeg")], body).into_response())
}

pub async fn upload(form: model::FileUpload) -> AppResponse {
    let temp_path_str = server_image::temp();
    let temp_path = server_image::relative_path(&temp_path_str)?;
    fs::rename(form.file.path, temp_path)?;
    Response::text(temp_path_str)
}


/// 显示图片
pub fn show_admin_image(path: String) -> String {
    if path.is_empty() {
        "/static/img/no_img.png?".to_string()
    } else {
        format!("/admin/image?path={}", path)
    }
}

mod model {
    use serde::{Deserialize, Serialize};
    use axum_multipart_file::MultipartFile;
    use axum_request_macros::RequestForm;

    #[derive(Debug, Default, Serialize, Deserialize, RequestForm)]
    pub struct FileUpload {
        pub file: MultipartFile,
    }

    #[derive(Debug, Serialize, Deserialize, RequestForm)]
    #[serde(rename_all = "camelCase")]
    pub struct ShowImage {
        /// 文件路径
        pub path: String,

        ///最大边宽
        pub max_size: u32,

        ///图片宽
        pub width: u32,

        ///图片高
        pub height: u32,
    }
}
