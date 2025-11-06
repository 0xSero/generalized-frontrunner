//! Metrics and health check endpoints

use axum::{
    response::IntoResponse,
    routing::get,
    Router,
};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use tracing::{error, info};

/// Serve Prometheus metrics
pub async fn serve_metrics(port: u16) {
    let app = Router::new().route("/metrics", get(metrics_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!(port = port, "Starting metrics server");

    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        error!(error = %e, "Metrics server error");
    }
}

/// Metrics handler
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!(error = %e, "Failed to encode metrics");
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to encode metrics".to_string(),
        );
    }

    let response = String::from_utf8(buffer).unwrap_or_else(|_| String::from("Error"));

    (axum::http::StatusCode::OK, response)
}

/// Serve health check endpoint
pub async fn serve_health_check(port: u16) {
    let app = Router::new().route("/health", get(health_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!(port = port, "Starting health check server");

    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        error!(error = %e, "Health check server error");
    }
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}
