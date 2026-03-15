use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Strip internal prefixes from ServerFnError messages for user-friendly display.
pub fn clean_error(e: &ServerFnError) -> String {
    let raw = e.to_string();
    raw.strip_prefix("error running server function: ")
        .or_else(|| raw.strip_prefix("ServerFnError: "))
        .unwrap_or(&raw)
        .to_string()
}

#[cfg(feature = "ssr")]
pub mod oauth;
pub mod otp;
pub mod password;
#[cfg(feature = "ssr")]
pub mod session;
#[cfg(feature = "ssr")]
mod validation;
#[cfg(feature = "ssr")]
pub use validation::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OtpResult {
    NewAccount,
    Existing,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Athlete,
    Coach,
    Admin,
}

impl UserRole {
    pub fn rank(&self) -> u8 {
        match self {
            UserRole::Athlete => 0,
            UserRole::Coach => 1,
            UserRole::Admin => 2,
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Athlete => write!(f, "athlete"),
            UserRole::Coach => write!(f, "coach"),
            UserRole::Admin => write!(f, "admin"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub gender: Option<String>,
}

impl AuthUser {
    pub fn initials(&self) -> String {
        self.display_name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }

    /// Returns the best available user identifier (email, phone, or fallback).
    pub fn identifier(&self) -> &str {
        self.email
            .as_deref()
            .or(self.phone.as_deref())
            .unwrap_or("")
    }
}

#[server]
pub async fn get_me() -> Result<Option<AuthUser>, ServerFnError> {
    let result = session::get_current_user().await;
    match &result {
        Ok(Some(u)) => tracing::info!("get_me: authenticated as {}", u.identifier()),
        Ok(None) => tracing::info!("get_me: no session found"),
        Err(e) => tracing::warn!("get_me: error: {}", e),
    }
    result
}
