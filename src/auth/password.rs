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

    leptos_axum::redirect("/");
    Ok(())
}

#[server]
pub async fn register_with_password(
    name: String,
    email: String,
    password: String,
) -> Result<(), ServerFnError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    // Validate inputs
    let name = name.trim().to_string();
    let email = email.trim().to_lowercase();

    if name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }
    if name.len() > 100 {
        return Err(ServerFnError::new("Name is too long"));
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(ServerFnError::new("Invalid email address"));
    }
    if password.len() < 8 {
        return Err(ServerFnError::new("Password must be at least 8 characters"));
    }
    if password.len() > 128 {
        return Err(ServerFnError::new("Password is too long"));
    }

    let pool = crate::db::db().await?;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .to_string();

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let role = if count.0 == 0 { "admin" } else { "athlete" };

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

    leptos_axum::redirect("/");
    Ok(())
}
