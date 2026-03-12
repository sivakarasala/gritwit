use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

pub async fn upload_video(
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    // Ensure videos directory exists
    let upload_dir = std::path::Path::new("public/videos");
    tokio::fs::create_dir_all(upload_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create upload dir: {}", e)))?;

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

        // Validate content type
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        if !content_type.starts_with("video/") {
            return Err((StatusCode::BAD_REQUEST, "Only video files are allowed".into()));
        }

        // Generate unique filename
        let ext = std::path::Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");
        let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
        let filepath = upload_dir.join(&filename);

        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e)))?;

        // Limit to 50MB
        if data.len() > 50 * 1024 * 1024 {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, "Video must be under 50MB".into()));
        }

        tokio::fs::write(&filepath, &data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {}", e)))?;

        let url = format!("/videos/{}", filename);
        return Ok(Json(UploadResponse { url }));
    }

    Err((StatusCode::BAD_REQUEST, "No video field found".into()))
}
