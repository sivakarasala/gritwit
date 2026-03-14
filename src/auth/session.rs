use super::{AuthUser, UserRole};
use leptos::prelude::*;
use tower_sessions::Session;

const USER_ID_KEY: &str = "user_id";

pub async fn get_session() -> Result<Session, ServerFnError> {
    let session: Session = leptos_axum::extract()
        .await
        .map_err(|e| ServerFnError::new(format!("Session extraction failed: {}", e)))?;
    Ok(session)
}

pub async fn get_current_user() -> Result<Option<AuthUser>, ServerFnError> {
    let session = get_session().await?;
    let user_id: Option<String> = session
        .get(USER_ID_KEY)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    tracing::info!(
        "get_current_user: session id={:?}, user_id={:?}",
        session.id(),
        user_id
    );

    let Some(uid) = user_id else {
        return Ok(None);
    };

    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = uid
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;

    let user = match crate::db::get_user_by_id(&pool, user_uuid).await {
        Ok(u) => Some(u),
        Err(e) => {
            tracing::error!("get_user_by_id failed: {:?}", e);
            None
        }
    };
    Ok(user)
}

pub async fn require_auth() -> Result<AuthUser, ServerFnError> {
    get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Unauthorized"))
}

pub async fn require_role(min_role: UserRole) -> Result<AuthUser, ServerFnError> {
    let user = require_auth().await?;
    if user.role.rank() >= min_role.rank() {
        Ok(user)
    } else {
        Err(ServerFnError::new("Insufficient permissions"))
    }
}

pub async fn set_user_id(session: &Session, user_id: &str) -> Result<(), ServerFnError> {
    session
        .insert(USER_ID_KEY, user_id.to_string())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
