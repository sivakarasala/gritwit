#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::routing::get;
    use axum::Router;
    use gritwit::app::*;
    use gritwit::auth::oauth::{self, OAuthState};
    use gritwit::configuration;
    use gritwit::routes::{health_check, ApiDoc};
    use gritwit::telemetry::{get_subscriber, init_subscriber};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::postgres::PgPoolOptions;
    use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
    use tower_http::trace::TraceLayer;
    use tower_sessions::cookie::SameSite;
    use tower_sessions::{Expiry, SessionManagerLayer};
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    dotenvy::dotenv().ok();

    let subscriber = get_subscriber("gritwit".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let app_config = configuration::get_configuration().expect("Failed to read configuration");

    let pool = PgPoolOptions::new().connect_lazy_with(app_config.database.connection_options());

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Could not run database migrations");

    // Session store
    let session_store = tower_sessions_sqlx_store::PostgresStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("Could not run session store migration");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(tower_sessions::cookie::time::Duration::hours(24)));

    // OAuth client
    let oauth_client = oauth::build_oauth_client(&app_config.oauth);
    let oauth_state = OAuthState {
        client: oauth_client,
        pool: pool.clone(),
    };

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let api_routes = Router::new().route("/health_check", get(health_check));

    let auth_routes = Router::new()
        .route("/auth/google/login", get(oauth::google_login))
        .route("/auth/google/callback", get(oauth::google_callback))
        .route("/auth/logout", get(oauth::logout))
        .with_state(oauth_state);

    let app = Router::new()
        .nest("/api/v1", api_routes)
        .merge(auth_routes)
        .merge(SwaggerUi::new("/api/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                move || provide_context(pool.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(session_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let request_id = request
                        .headers()
                        .get("x-request-id")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = %request_id,
                        status = tracing::field::Empty,
                        latency_ms = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     span: &tracing::Span| {
                        span.record("status", response.status().as_u16());
                        span.record("latency_ms", latency.as_millis() as u64);
                        tracing::info!("response");
                    },
                ),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .with_state(leptos_options);

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
