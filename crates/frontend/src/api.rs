// crates/frontend/src/api.rs
//
// HTTP client functions — call the apqp-service REST API via gloo-net.
// All functions are async and return Result<T, String>.

use gloo_net::http::Request;
use shared::models::{
    api::{ApiResponse, PagedResponse},
    task::{CreateTaskRequest, Task, TaskSummary, UpdateTaskRequest},
};

// Base URL — in dev points to apqp-service directly (no gateway yet)
const BASE_URL: &str = "/api";

// Dev tenant header — replace with JWT-derived value later
const TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";
const ACTOR_ID:  &str = "dev-user";

fn default_headers(req: gloo_net::http::RequestBuilder)
                   -> gloo_net::http::RequestBuilder
{
    req.header("X-Tenant-ID", TENANT_ID)
        .header("X-Actor-ID",  ACTOR_ID)
}

// ── Task endpoints ────────────────────────────────────────────

pub async fn list_tasks(
    status: Option<String>,
    page:   u32,
) -> Result<PagedResponse<TaskSummary>, String> {
    let mut url = format!("{BASE_URL}/tasks?page={page}&page_size=25");
    if let Some(s) = status {
        url.push_str(&format!("&progress_status={s}"));
    }

    let res = default_headers(Request::get(&url))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err(format!("HTTP {}", res.status()));
    }

    res.json::<PagedResponse<TaskSummary>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_task(id: &str) -> Result<Task, String> {
    let res = default_headers(Request::get(&format!("{BASE_URL}/tasks/{id}")))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        return Err(format!("HTTP {}", res.status()));
    }

    let envelope = res.json::<ApiResponse<Task>>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(envelope.data)
}

pub async fn create_task(req: CreateTaskRequest) -> Result<Task, String> {
    let body = serde_json::to_string(&req).map_err(|e| e.to_string())?;

    let res = default_headers(
        Request::post(&format!("{BASE_URL}/tasks"))
            .header("Content-Type", "application/json")
    )
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err: serde_json::Value = res.json().await.unwrap_or_default();
        return Err(err["error"].as_str().unwrap_or("Unknown error").to_string());
    }

    let envelope = res.json::<ApiResponse<Task>>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(envelope.data)
}

pub async fn update_task_status(
    id:     &str,
    status: &str,
) -> Result<Task, String> {
    let body = serde_json::json!({ "status": status }).to_string();

    let res = default_headers(
        Request::patch(&format!("{BASE_URL}/tasks/{id}/status"))
            .header("Content-Type", "application/json")
    )
        .body(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.ok() {
        let err: serde_json::Value = res.json().await.unwrap_or_default();
        return Err(err["error"].as_str().unwrap_or("Unknown error").to_string());
    }

    let envelope = res.json::<ApiResponse<Task>>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(envelope.data)
}
