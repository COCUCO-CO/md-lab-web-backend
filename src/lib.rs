//! MD Lab web backend — application library.
//!
//! The router lives here (not in `main.rs`) so integration tests can drive it
//! without binding a port.

use axum::routing::get;
use axum::Json;
use axum::{response::IntoResponse, Router};
use serde::Serialize;

/// Initialize Sentry from the environment, returning the client guard.
///
/// The guard must stay alive for the process lifetime — dropping it flushes
/// pending events and disables further capture. Returns `None` when no DSN is
/// configured so local development and tests run with zero overhead.
pub fn init_sentry() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").ok()?;
    if dsn.is_empty() {
        return None;
    }
    let environment = std::env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "production".into());
    // Same release identity the /api/version endpoint reports, so a Sentry
    // issue maps straight back to the deployed commit.
    let release = format!(
        "{}+{}",
        option_env!("MDLAB_VERSION").unwrap_or("dev"),
        option_env!("GIT_SHA").unwrap_or("unknown")
    );
    let guard = sentry::init(
        sentry::ClientOptions::new()
            .dsn(&dsn)
            .release(release)
            .environment(environment)
            .traces_sample_rate(1.0),
    );
    Some(guard)
}

/// Build the application router.
pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/version", get(version))
        .route("/api/pipeline", get(pipeline))
}

/// `GET /api/health` — liveness probe used by the ALB target group and ECS.
#[tracing::instrument(skip_all)]
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

#[derive(Serialize)]
struct VersionInfo {
    name: &'static str,
    version: String,
    git_sha: String,
}

/// `GET /api/version` — release traceability for the running binary.
///
/// `MDLAB_VERSION` and `GIT_SHA` are baked in at build time (Docker build
/// args); local/dev builds report `dev`/`unknown`.
#[tracing::instrument(skip_all)]
async fn version() -> impl IntoResponse {
    Json(VersionInfo {
        name: env!("CARGO_PKG_NAME"),
        version: option_env!("MDLAB_VERSION").unwrap_or("dev").to_string(),
        git_sha: option_env!("GIT_SHA").unwrap_or("unknown").to_string(),
    })
}

#[derive(Serialize)]
struct Phase {
    number: &'static str,
    title: &'static str,
    description: &'static str,
}

const PHASES: [Phase; 5] = [
    Phase {
        number: "01",
        title: "System build",
        description: "Prepare molecular systems — proteins, ligands, and solvent boxes — ready for simulation.",
    },
    Phase {
        number: "02",
        title: "Equilibration",
        description: "Relax each system step by step so production runs start from a stable state.",
    },
    Phase {
        number: "03",
        title: "Production MD",
        description: "Run long GROMACS simulations in managed campaigns with tracked parameters and resources.",
    },
    Phase {
        number: "04",
        title: "Analysis",
        description: "Extract trajectories, observables, and metrics from every run with reproducible pipelines.",
    },
    Phase {
        number: "05",
        title: "Reporting",
        description: "Ship artifacts, decision records, and reports that make each campaign auditable.",
    },
];

/// `GET /api/pipeline` — the five-phase MD pipeline, in order.
#[tracing::instrument(skip_all)]
async fn pipeline() -> impl IntoResponse {
    Json(serde_json::json!({ "phases": PHASES }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Both tests mutate SENTRY_DSN in the process-wide environment, so they
    /// must not run in parallel with each other.
    static DSN_LOCK: Mutex<()> = Mutex::new(());

    /// No DSN configured (local dev / tests) -> Sentry stays fully disabled.
    #[test]
    fn sentry_init_is_disabled_without_dsn() {
        let _guard = DSN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("SENTRY_DSN");
        assert!(init_sentry().is_none());
    }

    /// A syntactically valid DSN -> a live client guard is created. The guard
    /// drop flushes (no-op queue) and disables the client again.
    #[test]
    fn sentry_init_enables_client_with_valid_dsn() {
        let _guard = DSN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("SENTRY_DSN", "https://key@o0.ingest.sentry.io/1");
        let client = init_sentry();
        assert!(client.is_some());
        drop(client);
        std::env::remove_var("SENTRY_DSN");
    }
}
