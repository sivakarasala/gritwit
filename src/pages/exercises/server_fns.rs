#[cfg(feature = "ssr")]
use crate::auth::UserRole;
use crate::db::Exercise;
use leptos::prelude::*;

#[server]
pub async fn list_exercises() -> Result<Vec<Exercise>, ServerFnError> {
    let pool = crate::db::db().await?;
    crate::db::list_exercises_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn create_exercise(
    name: String,
    category: String,
    movement_type: String,
    description: String,
    demo_video_url: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let mt = if movement_type.is_empty() {
        None
    } else {
        Some(movement_type.as_str())
    };
    let desc = if description.is_empty() {
        None
    } else {
        Some(description.as_str())
    };
    let video = if demo_video_url.is_empty() {
        None
    } else {
        Some(demo_video_url.as_str())
    };
    crate::db::create_exercise_db(
        &pool,
        &name,
        &category,
        mt,
        &[],
        desc,
        video,
        Some(user_uuid),
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn update_exercise(
    id: String,
    name: String,
    category: String,
    movement_type: String,
    description: String,
    demo_video_url: String,
) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let mt = if movement_type.is_empty() {
        None
    } else {
        Some(movement_type.as_str())
    };
    let desc = if description.is_empty() {
        None
    } else {
        Some(description.as_str())
    };
    let video = if demo_video_url.is_empty() {
        None
    } else {
        Some(demo_video_url.as_str())
    };
    crate::db::update_exercise_db(&pool, uuid, &name, &category, mt, desc, video)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
pub async fn delete_exercise(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_exercise_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
