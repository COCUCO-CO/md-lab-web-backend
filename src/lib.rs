//! MD Lab web backend — application library.
//!
//! The router lives here (not in `main.rs`) so integration tests can drive it
//! without binding a port.

use axum::routing::get;
use axum::Json;
use axum::{response::IntoResponse, Router};
use serde::Serialize;

/// Build the application router.
pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/version", get(version))
        .route("/api/pipeline", get(pipeline))
}

/// `GET /api/health` — liveness probe used by the ALB target group and ECS.
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
async fn pipeline() -> impl IntoResponse {
    Json(serde_json::json!({ "phases": PHASES }))
}
