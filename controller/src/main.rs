use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // healthcheck endpoint
        .route("/healthz", get(healthz));

    // run app with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    _ = axum::serve(listener, app).await;
}

/// Healthcheck handler.
async fn healthz() -> &'static str {
    "Ok"
}
