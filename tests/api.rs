//! Integration tests for the MD Lab web backend API.
//!
//! These define the public contract of the service:
//! - GET /api/health   -> 200 {"status":"ok"}          (ALB/ECS health checks)
//! - GET /api/version  -> 200 {name, version, git_sha} (release traceability)
//! - GET /api/pipeline -> 200 {phases: [5 phases]}     (MD pipeline data)
//! - anything else     -> 404                           (the SPA owns non-/api paths)

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt; // for `oneshot`

/// Build a request and run it against the application router.
async fn send(req: Request<Body>) -> (StatusCode, Value) {
    let response = md_lab_web_backend::router()
        .oneshot(req)
        .await
        .expect("request failed");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body collection failed")
        .to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

fn get(path: &str) -> Request<Body> {
    Request::builder().uri(path).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn health_returns_ok() {
    let (status, body) = send(get("/api/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!({ "status": "ok" }));
}

#[tokio::test]
async fn version_reports_name_version_and_sha() {
    let (status, body) = send(get("/api/version")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "md-lab-web-backend");
    assert!(
        body["version"].is_string() && !body["version"].as_str().unwrap().is_empty(),
        "version must be a non-empty string"
    );
    assert!(
        body["git_sha"].is_string() && !body["git_sha"].as_str().unwrap().is_empty(),
        "git_sha must be a non-empty string"
    );
}

#[tokio::test]
async fn pipeline_returns_the_five_md_phases_in_order() {
    let (status, body) = send(get("/api/pipeline")).await;
    assert_eq!(status, StatusCode::OK);

    let phases = body["phases"].as_array().expect("phases must be an array");
    assert_eq!(phases.len(), 5, "the MD pipeline has exactly five phases");

    for phase in phases {
        assert!(phase["number"].is_string(), "phase.number missing");
        assert!(phase["title"].is_string() && !phase["title"].as_str().unwrap().is_empty());
        assert!(
            phase["description"].is_string() && !phase["description"].as_str().unwrap().is_empty()
        );
    }

    let titles: Vec<&str> = phases
        .iter()
        .map(|p| p["title"].as_str().unwrap())
        .collect();
    assert_eq!(
        titles,
        vec![
            "System build",
            "Equilibration",
            "Production MD",
            "Analysis",
            "Reporting"
        ]
    );
}

#[tokio::test]
async fn root_path_is_not_part_of_the_api() {
    let (status, _) = send(get("/")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn unknown_api_path_returns_404() {
    let (status, _) = send(get("/api/does-not-exist")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
