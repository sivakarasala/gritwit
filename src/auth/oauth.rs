use axum::{
    extract::{Query, State},
    response::Redirect,
};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::configuration::OAuthSettings;

const CSRF_STATE_KEY: &str = "oauth_csrf_state";

// Type alias for the fully-configured oauth2 client (with auth_uri + token_uri set)
type ConfiguredClient = oauth2::Client<
    oauth2::basic::BasicErrorResponse,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardRevocableToken,
    oauth2::basic::BasicRevocationErrorResponse,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
>;

#[derive(Clone)]
pub struct OAuthState {
    pub client: ConfiguredClient,
    pub pool: sqlx::PgPool,
}

pub fn build_oauth_client(config: &OAuthSettings) -> ConfiguredClient {
    use secrecy::ExposeSecret;

    let client_id = config.google_client_id.expose_secret().clone();

    oauth2::basic::BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(
            config.google_client_secret.expose_secret().clone(),
        ))
        .set_auth_uri(
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                .expect("Invalid auth URL"),
        )
        .set_token_uri(
            TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .expect("Invalid token URL"),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_url.clone()).expect("Invalid redirect URL"),
        )
}

pub async fn google_login(
    State(state): State<OAuthState>,
    session: Session,
) -> Result<Redirect, axum::http::StatusCode> {
    let (auth_url, csrf_token): (oauth2::url::Url, CsrfToken) = state
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    let csrf_secret: String = csrf_token.secret().clone();
    session
        .insert(CSRF_STATE_KEY, csrf_secret)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::temporary(auth_url.as_str()))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    sub: String,
    email: String,
    name: String,
    picture: Option<String>,
}

pub async fn google_callback(
    State(state): State<OAuthState>,
    session: Session,
    Query(params): Query<CallbackParams>,
) -> Result<Redirect, axum::http::StatusCode> {
    // Validate CSRF
    let stored_state: Option<String> = session
        .get(CSRF_STATE_KEY)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(stored) = stored_state else {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    };

    if stored != params.state {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    session
        .remove::<String>(CSRF_STATE_KEY)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Exchange code for token
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let token_request = state
        .client
        .exchange_code(AuthorizationCode::new(params.code));
    let token_result = token_request
        .request_async(&http_client)
        .await
        .map_err(|e| {
            tracing::error!("Token exchange failed: {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Fetch user info from Google
    let userinfo: GoogleUserInfo = http_client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .json()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if first user (becomes admin)
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let role = if user_count.0 == 0 {
        "admin"
    } else {
        "athlete"
    };

    // Upsert user
    let user_id: (uuid::Uuid,) = sqlx::query_as(
        r#"INSERT INTO users (google_id, email, display_name, avatar_url, role)
           VALUES ($1, $2, $3, $4, $5::user_role)
           ON CONFLICT (google_id) DO UPDATE SET
               email = EXCLUDED.email,
               display_name = EXCLUDED.display_name,
               avatar_url = EXCLUDED.avatar_url,
               updated_at = now()
           RETURNING id"#,
    )
    .bind(&userinfo.sub)
    .bind(&userinfo.email)
    .bind(&userinfo.name)
    .bind(&userinfo.picture)
    .bind(role)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Set session
    session
        .insert("user_id", user_id.0.to_string())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::temporary("/"))
}

pub async fn logout(session: Session) -> Redirect {
    let _ = session.flush().await;
    Redirect::temporary("/")
}
