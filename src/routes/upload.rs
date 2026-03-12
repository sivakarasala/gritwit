use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;
use std::sync::Arc;

use crate::storage::StorageBackend;

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

pub async fn upload_video(
    State(storage): State<Arc<StorageBackend>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "video" {
            continue;
        }

        let original_name = field
            .file_name()
            .unwrap_or("video.mp4")
            .to_string();

        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        if !content_type.starts_with("video/") {
            return Err((StatusCode::BAD_REQUEST, "Only video files are allowed".into()));
        }

        let ext = std::path::Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");
        let key = format!("{}.{}", uuid::Uuid::new_v4(), ext);

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e)))?;

        if data.len() > 50 * 1024 * 1024 {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, "Video must be under 50MB".into()));
        }

        let url = storage
            .upload(&key, &data, &content_type)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        return Ok(Json(UploadResponse { url }));
    }

    Err((StatusCode::BAD_REQUEST, "No video field found".into()))
}
