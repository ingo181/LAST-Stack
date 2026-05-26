// ─────────────────────────────────────────────────────────────
// apqp-service/src/handlers/health.rs
// ─────────────────────────────────────────────────────────────

// pub mod health — paste this content into handlers/health.rs

use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::state::AppState;

/// GET /health — liveness probe (always 200 if process is up)
pub async fn liveness() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

/// GET /ready — readiness probe (checks SurrealDB connection)
pub async fn readiness(State(state): State<AppState>)
                       -> (StatusCode, Json<Value>)
{
    match state.db.health().await {
        Ok(_) => (StatusCode::OK,
                  Json(json!({ "status": "ready", "db": "ok" }))),
        Err(e) => (StatusCode::SERVICE_UNAVAILABLE,
                   Json(json!({ "status": "not_ready", "db": e.to_string() }))),
    }
}
