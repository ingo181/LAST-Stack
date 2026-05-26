// apqp-service/src/state.rs
//
// Shared application state — cloned into every Axum handler via
// axum::extract::State<AppState>. All fields are Arc-wrapped
// internally by SurrealDB's client, so Clone is cheap.

use surrealdb::{engine::remote::ws::Client, Surreal};

#[derive(Clone)]
pub struct AppState {
    pub db: Surreal<Client>,
}

impl AppState {
    pub fn new(db: Surreal<Client>) -> Self {
        Self { db }
    }
}

// ── TenantContext ─────────────────────────────────────────────
// Extracted from JWT by the API Gateway and forwarded as the
// X-Tenant-ID header. The apqp-service reads it here and
// selects the correct SurrealDB database for every request.
//
// In production the Gateway validates the JWT; the service only
// reads the already-trusted header. For the MVP we use a simple
// extractor that defaults to a dev tenant if the header is absent.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub actor_id:  String,   // JWT sub — user performing the action
}

impl TenantContext {
    pub fn db_name(&self) -> String {
        format!("tenant_{}", self.tenant_id.simple())
    }
}

impl<S> FromRequestParts<S> for TenantContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection>
    {
        // Gateway sets X-Tenant-ID after JWT validation.
        // For local dev without Gateway, fall back to a fixed UUID.
        let tenant_id = parts
            .headers
            .get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(|| {
                // Dev-only fallback — deterministic UUID for local testing
                Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                    .expect("static UUID is valid")
            });

        // X-Actor-ID = JWT sub forwarded by Gateway
        let actor_id = parts
            .headers
            .get("x-actor-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("dev-user")
            .to_owned();

        Ok(TenantContext { tenant_id, actor_id })
    }
}
