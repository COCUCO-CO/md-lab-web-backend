//! MD Lab web backend — HTTP server entrypoint.

use std::net::SocketAddr;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // Keep the guard alive: dropping it flushes pending events and disables
    // capture. None when SENTRY_DSN is unset (local dev / tests).
    let _sentry_guard = md_lab_web_backend::init_sentry();

    // Registry with two layers: the human-readable fmt output (filtered by
    // RUST_LOG) and the Sentry tracing layer, which records spans as Sentry
    // spans/transactions and error-level events as issues.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            ),
        )
        .with(sentry::integrations::tracing::layer())
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!(
        %addr,
        version = option_env!("MDLAB_VERSION").unwrap_or("dev"),
        "starting md-lab-web-backend"
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");
    axum::serve(listener, md_lab_web_backend::router())
        .await
        .expect("server error");
}
