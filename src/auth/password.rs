use leptos::prelude::*;

#[server]
pub async fn login_with_password(email: String, password: String) -> Result<(), ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let pool = crate::db::db().await?;

    #[derive(sqlx::FromRow)]
    struct Row {
        id: uuid::Uuid,
        password_hash: Option<String>,
    }

    let row: Option<Row> = sqlx::query_as("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let row = row.ok_or_else(|| ServerFnError::new("Invalid email or password"))?;

    let hash = row
        .password_hash
        .ok_or_else(|| ServerFnError::new("Invalid email or password"))?;

    let parsed = PasswordHash::new(&hash).map_err(|e| ServerFnError::new(e.to_string()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ServerFnError::new("Invalid email or password"))?;

    let session = super::session::get_session().await?;
    super::session::set_user_id(&session, &row.id.to_string()).await?;

    Ok(())
}

#[server]
pub async fn register_with_password(
    name: String,
    email: String,
    password: String,
) -> Result<(), ServerFnError> {
    use super::{
        default_role_for_new_user, hash_password, validate_email, validate_name, validate_password,
    };

    let name = validate_name(&name)?;
    let email = validate_email(&email)?.ok_or_else(|| ServerFnError::new("Email is required"))?;
    validate_password(&password)?;

    let pool = crate::db::db().await?;
    let hash = hash_password(&password)?;
    let role = default_role_for_new_user(&pool).await?;

    let user_id: (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO users (email, display_name, password_hash, role)
           VALUES ($1, $2, $3, $4::user_role)
           RETURNING id"#,
    )
    .bind(&email)
    .bind(&name)
    .bind(&hash)
    .bind(role)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique") || msg.contains("duplicate") {
            ServerFnError::new("An account with this email already exists")
        } else {
            ServerFnError::new("Failed to create account")
        }
    })?;

    let session = super::session::get_session().await?;
    super::session::set_user_id(&session, &user_id.0.to_string()).await?;

    Ok(())
}
