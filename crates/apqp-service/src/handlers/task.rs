// ─────────────────────────────────────────────────────────────
// apqp-service/src/handlers/task.rs
// ─────────────────────────────────────────────────────────────

// pub mod task — paste this content into handlers/task.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use shared::{
    errors::AppError,
    models::{
        api::{ApiResponse, PagedResponse, Pagination},
        task::{
            CreateTaskRequest, ProgressStatus, RiskStatus,
            Task, TaskHistory, TaskSummary, UpdateTaskRequest,
        },
    },
};

use crate::{db, state::{AppState, TenantContext}};

#[derive(Debug, serde::Deserialize)]
pub struct StatusTransitionRequest {
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct RiskUpdateRequest {
    pub risk:   String,
    pub reason: Option<String>,
}


// ── Error → HTTP response ──────────────────────────────────────
//
// Orphan rule: we cannot impl IntoResponse for AppError (foreign type)
// directly. We wrap it in a local newtype instead.

pub struct ApiError(AppError);

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.0.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = Json(serde_json::json!({ "error": self.0.to_string() }));
        (status, body).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self { ApiError(e) }
}

// ── List ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TaskFilter {
    pub progress_status: Option<String>,
    pub assigned_to:     Option<String>,
    pub party_id:        Option<String>,
    #[serde(flatten)]
    pub pagination:      Pagination,
}

pub async fn list(
    State(state):  State<AppState>,
    tenant:        TenantContext,
    Query(filter): Query<TaskFilter>,
) -> Result<Json<PagedResponse<TaskSummary>>, ApiError> {
    // Switch to tenant database
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let status = filter.progress_status
        .as_deref()
        .map(|s| s.parse::<ProgressStatus>())
        .transpose()
        .map_err(|e| AppError::Validation(e))?;

    let limit  = filter.pagination.page_size;
    let offset = (filter.pagination.page.saturating_sub(1)) * limit;

    let items = db::list_tasks(
        &state.db,
        status,
        filter.assigned_to.clone(),
        filter.party_id.clone(),
        limit,
        offset,
    )
        .await?;

    Ok(Json(PagedResponse {
        total: items.len() as u64, // TODO: COUNT(*) query
        items,
        page:      filter.pagination.page,
        page_size: limit,
    }))
}

// ── Get by ID ─────────────────────────────────────────────────

pub async fn get_by_id(
    State(state): State<AppState>,
    tenant:       TenantContext,
    Path(id):     Path<String>,
) -> Result<Json<ApiResponse<Task>>, ApiError> {
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| ApiError(AppError::Database(e.to_string())))?;
    let task = db::get_task(&state.db, &id).await.map_err(ApiError)?;
    Ok(Json(ApiResponse::ok(task)))
}

pub async fn create(
    State(state): State<AppState>,
    tenant:       TenantContext,
    Json(req):    Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Task>>), ApiError> {
    if req.subject.trim().is_empty() {
        return Err(ApiError(AppError::Validation("subject must not be empty".into())));
    }
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| ApiError(AppError::Database(e.to_string())))?;
    let task = db::create_task(&state.db, req, &tenant.actor_id, tenant.tenant_id)
        .await.map_err(ApiError)?;
    Ok((StatusCode::CREATED, Json(ApiResponse::ok(task))))
}

pub async fn update(
    State(state): State<AppState>,
    tenant:       TenantContext,
    Path(id):     Path<String>,
    Json(req):    Json<UpdateTaskRequest>,
) -> Result<Json<ApiResponse<Task>>, ApiError> {
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| ApiError(AppError::Database(e.to_string())))?;
    let task = db::update_task(&state.db, &id, req, &tenant.actor_id, tenant.tenant_id)
        .await.map_err(ApiError)?;
    Ok(Json(ApiResponse::ok(task)))
}

pub async fn transition_status(
    State(state): State<AppState>,
    tenant:       TenantContext,
    Path(id):     Path<String>,
    Json(req):    Json<StatusTransitionRequest>,
) -> Result<Json<ApiResponse<Task>>, ApiError> {
    let new_status = req.status.parse::<ProgressStatus>()
        .map_err(|e| ApiError(AppError::Validation(e)))?;
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| ApiError(AppError::Database(e.to_string())))?;
    let task = db::transition_status(&state.db, &id, new_status, &tenant.actor_id, tenant.tenant_id)
        .await.map_err(ApiError)?;
    Ok(Json(ApiResponse::ok(task)))
}

pub async fn update_risk(
    State(state): State<AppState>,
    tenant:       TenantContext,
    Path(id):     Path<String>,
    Json(req):    Json<RiskUpdateRequest>,
) -> Result<Json<ApiResponse<Task>>, ApiError> {
    let new_risk = req.risk.parse::<RiskStatus>()
        .map_err(|e| ApiError(AppError::Validation(e)))?;
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| ApiError(AppError::Database(e.to_string())))?;
    let task = db::update_risk(&state.db, &id, new_risk, req.reason, &tenant.actor_id, tenant.tenant_id)
        .await.map_err(ApiError)?;
    Ok(Json(ApiResponse::ok(task)))
}

pub async fn soft_delete(
    State(state): State<AppState>,
    tenant:       TenantContext,
    Path(id):     Path<String>,
) -> Result<StatusCode, ApiError> {
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| ApiError(AppError::Database(e.to_string())))?;
    db::soft_delete(&state.db, &id, &tenant.actor_id, tenant.tenant_id)
        .await.map_err(ApiError)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn history(
    State(state): State<AppState>,
    tenant:       TenantContext,
    Path(id):     Path<String>,
) -> Result<Json<ApiResponse<Vec<TaskHistory>>>, ApiError> {
    state.db.use_db(tenant.db_name()).await
        .map_err(|e| ApiError(AppError::Database(e.to_string())))?;
    let entries = db::get_history(&state.db, &id).await.map_err(ApiError)?;
    Ok(Json(ApiResponse::ok(entries)))
}
