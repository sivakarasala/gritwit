#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::routing::get;
    use axum::Router;
    use gritwit::app::*;
    use gritwit::configuration;
    use gritwit::routes::{health_check, ApiDoc};
    use gritwit::telemetry::{get_subscriber, init_subscriber};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::postgres::PgPoolOptions;
    use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
    use tower_http::trace::TraceLayer;
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    let _ = dotenvy::dotenv();

    let subscriber = get_subscriber("gritwit".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let app_config = configuration::get_configuration().expect("Failed to read configuration");

    let pool = PgPoolOptions::new().connect_lazy_with(app_config.database.connection_options());

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Could not run database migrations");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let api_routes = Router::new().route("/health_check", get(health_check));

    let app = Router::new()
        .nest("/api/v1", api_routes)
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
