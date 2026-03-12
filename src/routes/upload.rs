use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;
use std::sync::Arc;
use tower_sessions::Session;

use crate::storage::StorageBackend;

/// Shared state for the upload route: storage backend + DB pool for auth.
#[derive(Clone)]
pub struct UploadState {
    pub storage: Arc<StorageBackend>,
    pub pool: sqlx::PgPool,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
}

/// Allowed video extensions.
const ALLOWED_EXTENSIONS: &[&str] = &["mp4", "webm", "mov", "avi", "m4v"];

/// Max upload size: 100 MB.
const MAX_UPLOAD_BYTES: usize = 100 * 1024 * 1024;

/// Validate file magic bytes against known video container signatures.
fn is_valid_video_magic(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }

    // MP4 / M4V / MOV — ftyp box at offset 4
    if data[4..8] == *b"ftyp" {
        return true;
    }

    // WebM / MKV — EBML header
    if data[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
        return true;
    }

    // AVI — RIFF....AVI
    if data[0..4] == *b"RIFF" && data.len() >= 12 && data[8..12] == *b"AVI " {
        return true;
    }

    false
}

pub async fn upload_video(
    State(state): State<UploadState>,
    session: Session,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    // --- Auth guard: require a logged-in user ---
    let user_id: Option<String> = session
        .get("user_id")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let user_id = user_id.ok_or((StatusCode::UNAUTHORIZED, "Sign in to upload videos".into()))?;

    let user_uuid: uuid::Uuid = user_id
        .parse()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session".into()))?;

    // Verify user still exists
    let _user = crate::db::get_user_by_id(&state.pool, user_uuid)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "User not found".into()))?;

    // --- Process upload ---
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "video" {
            continue;
        }

        let original_name = field.file_name().unwrap_or("video.mp4").to_string();

        // Validate content type
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        if !content_type.starts_with("video/") {
            return Err((
                StatusCode::BAD_REQUEST,
                "Only video files are allowed".into(),
            ));
        }

        // Validate extension against allowlist
        let ext = std::path::Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "Unsupported file type '.{}'. Allowed: {}",
                    ext,
                    ALLOWED_EXTENSIONS.join(", ")
                ),
            ));
        }

        // Read bytes
        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read file: {}", e),
            )
        })?;

        // Size check
        if data.len() > MAX_UPLOAD_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                "Video must be under 100 MB".into(),
            ));
        }

        // Magic bytes validation — ensure the file is actually a video
        if !is_valid_video_magic(&data) {
            return Err((
                StatusCode::BAD_REQUEST,
                "File content does not match a supported video format".into(),
            ));
        }

        let key = format!("{}.{}", uuid::Uuid::new_v4(), ext);

        let url = state
            .storage
            .upload(&key, &data, &content_type)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        tracing::info!(
            user_id = %user_uuid,
            file = %original_name,
            size_bytes = data.len(),
            "video uploaded"
        );

        return Ok(Json(UploadResponse { url }));
    }

    Err((StatusCode::BAD_REQUEST, "No video field found".into()))
}
