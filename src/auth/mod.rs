use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
pub mod oauth;
#[cfg(feature = "ssr")]
pub mod session;

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
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
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
}

#[server]
pub async fn get_me() -> Result<Option<AuthUser>, ServerFnError> {
    session::get_current_user().await
}
