//! Generalized Frontrunner
//!
//! Main entry point for the frontrunner system

use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod orchestrator;
mod metrics;

use orchestrator::Orchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    init_tracing();

    info!("Starting Generalized Frontrunner v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = types::Config::load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    info!(
        environment = %config.environment.name,
        "Configuration loaded"
    );

    // Initialize database connection pool
    let db_pool = init_database(&config).await?;

    // Start metrics server
    let metrics_handle = tokio::spawn(metrics::serve_metrics(config.monitoring.metrics_port));

    // Start health check server
    let health_handle = tokio::spawn(metrics::serve_health_check(config.monitoring.health_check_port));

    // Create and run orchestrator
    let orchestrator = Orchestrator::new(config, db_pool).await?;

    // Run the orchestrator (this will run indefinitely)
    let orchestrator_result = orchestrator.run().await;

    // If orchestrator exits, clean up
    info!("Orchestrator shutting down");

    metrics_handle.abort();
    health_handle.abort();

    orchestrator_result
}

/// Initialize tracing/logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,frontrunner=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();
}

/// Initialize database connection pool
async fn init_database(config: &types::Config) -> Result<sqlx::PgPool> {
    info!(url = "[REDACTED]", "Connecting to database");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect_timeout(std::time::Duration::from_secs(
            config.database.connection_timeout_seconds,
        ))
        .connect(&config.database.url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    info!("Database connected");

    // Run migrations
    info!("Running database migrations");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

    info!("Database migrations complete");

    Ok(pool)
}
