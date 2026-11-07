use axum::{Router, routing::get};

/// Shared HTTP client.
mod http_client;
/// API routes.
mod routes;
/// Agent worker container methods.
mod worker;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // healthcheck endpoint
        .route("/healthz", get(routes::healthz::get))
        // request a code review
        .route(
            "/v1/request-code-review",
            get(routes::v1::request_code_review::get).post(routes::v1::request_code_review::post),
        );

    // run app with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    _ = axum::serve(listener, app).await;
}
