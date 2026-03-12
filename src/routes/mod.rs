mod health_check;
mod upload;

pub use health_check::health_check;
pub use upload::{upload_video, UploadState};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check::health_check,
    ),
    tags(
        (name = "v1", description = "API v1 endpoints")
    )
)]
pub struct ApiDoc;
