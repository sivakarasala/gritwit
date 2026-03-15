use leptos::prelude::ServerFnError;

/// Validate a display name: trimmed, non-empty, max 100 chars.
pub fn validate_name(name: &str) -> Result<String, ServerFnError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }
    if name.len() > 100 {
        return Err(ServerFnError::new("Name is too long"));
    }
    Ok(name)
}

/// Validate an email address: trimmed, lowercased, must contain @ and dot.
/// Returns None for empty input, Some(email) for valid input.
pub fn validate_email(email: &str) -> Result<Option<String>, ServerFnError> {
    let email = email.trim().to_lowercase();
    if email.is_empty() {
        return Ok(None);
    }
    if !email.contains('@') || !email.contains('.') {
        return Err(ServerFnError::new("Invalid email address"));
    }
    Ok(Some(email))
}

/// Validate password length: 8–128 characters.
pub fn validate_password(password: &str) -> Result<(), ServerFnError> {
    if password.len() < 8 {
        return Err(ServerFnError::new("Password must be at least 8 characters"));
    }
    if password.len() > 128 {
        return Err(ServerFnError::new("Password is too long"));
    }
    Ok(())
}

/// Hash a password with Argon2 + random salt.
pub fn hash_password(password: &str) -> Result<String, ServerFnError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Determine the role for a new user: first user is admin, rest are athletes.
pub async fn default_role_for_new_user(pool: &sqlx::PgPool) -> Result<&'static str, ServerFnError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(if count.0 == 0 { "admin" } else { "athlete" })
}
