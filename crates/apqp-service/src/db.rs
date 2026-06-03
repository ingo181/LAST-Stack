// apqp-service/src/db.rs
//
// SurrealDB 3.x repository for the Task aggregate.
// Key differences from 2.x:
//   • .bind() requires owned values — no &str (lifetime constraint)
//   • .take::<T>() requires T: SurrealValue — use serde_json::Value
//     as intermediate, then deserialise manually
//   • .select() still works for simple record lookups

use chrono::Utc;
use surrealdb::{engine::remote::ws::Client, Surreal};
use tracing::{debug, instrument};
use uuid::Uuid;

use shared::{
    errors::AppError,
    events::{TaskEvent, TaskStatusChanged, topics},
    models::task::{
        CreateTaskRequest, ProgressStatus, RiskStatus,
        Task, TaskHistory, TaskSummary, UpdateTaskRequest,
    },
};

type Result<T> = std::result::Result<T, AppError>;

fn db_err(e: impl std::fmt::Display) -> AppError {
    AppError::Database(e.to_string())
}

fn ser_err(e: impl std::fmt::Display) -> AppError {
    AppError::Serialisation(e.to_string())
}

/// Deserialise a SurrealDB query result into T via serde_json.
/// SurrealDB 3.x returns serde-compatible Values; we round-trip
/// through JSON to avoid the SurrealValue trait bound on our types.
fn take_one<T: serde::de::DeserializeOwned>(
    res: &mut surrealdb::IndexedResults,
    idx: usize,
) -> Result<Option<T>> {
    let val: Option<serde_json::Value> = res.take(idx).map_err(db_err)?;
    match val {
        None => Ok(None),
        Some(v) => serde_json::from_value(v).map(Some).map_err(ser_err),
    }
}

fn take_vec<T: serde::de::DeserializeOwned>(
    res: &mut surrealdb::IndexedResults,
    idx: usize,
) -> Result<Vec<T>> {
    let val: Vec<serde_json::Value> = res.take(idx).map_err(db_err)?;
    val.into_iter()
        .map(|v| serde_json::from_value(v).map_err(ser_err))
        .collect()
}

// ─────────────────────────────────────────────────────────────
// List
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db))]
pub async fn list_tasks(
    db:              &Surreal<Client>,
    progress_status: Option<ProgressStatus>,
    assigned_to:     Option<String>,
    party_id:        Option<String>,
    limit:           u32,
    offset:          u32,
) -> Result<Vec<TaskSummary>> {
    let mut conditions = vec!["deleted_at = NONE".to_owned()];
    if let Some(ref s) = progress_status {
        conditions.push(format!("progress_status = '{s}'"));
    }
    if let Some(ref a) = assigned_to {
        conditions.push(format!("assigned_to = contact:{a}"));
    }
    if let Some(ref p) = party_id {
        conditions.push(format!("party_id = party:{p}"));
    }

    let where_clause = conditions.join(" AND ");
    let query = format!(
        "SELECT id, subject, priority, progress_status, risk_status,
                completion, dates.planned_end AS planned_end,
                <string> assigned_to AS assigned_to,
                <string> party_id AS party_id
         FROM task WHERE {where_clause}
         ORDER BY dates.planned_end ASC
         LIMIT {limit} START {offset}"
    );
    debug!(query = %query, "list_tasks");

    let mut res = db.query(query).await.map_err(db_err)?;
    let raw: Vec<serde_json::Value> = res.take(0).map_err(db_err)?;
    tracing::info!("raw list result: {}", serde_json::to_string(&raw).unwrap_or_default());
    serde_json::from_value(serde_json::Value::Array(raw)).map_err(ser_err)
}

// ─────────────────────────────────────────────────────────────
// Get by ID
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db))]
pub async fn get_task(db: &Surreal<Client>, id: &str) -> Result<Task> {
    let mut res = db
        .query("SELECT * FROM type::record('task', $id)")
        .bind(("id".to_owned(), id.to_owned()))
        .await
        .map_err(db_err)?;

    take_one(&mut res, 0)?
        .ok_or_else(|| AppError::NotFound(format!("task:{id}")))
}

// ─────────────────────────────────────────────────────────────
// Create
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db, actor_id))]
pub async fn create_task(
    db:        &Surreal<Client>,
    req:       CreateTaskRequest,
    actor_id:  &str,
    tenant_id: Uuid,
) -> Result<Task> {
    let now = Utc::now();
    let dates_json = req.dates
        .map(|d| serde_json::to_value(d).map_err(ser_err))
        .transpose()?
        .unwrap_or_else(|| serde_json::json!({}));
    let ext_json   = serde_json::to_value(&req.external_ref).map_err(ser_err)?;

    let mut res = db
        .query(
            "CREATE task CONTENT {
                subject:         $subject,
                description:     $description,
                priority:        $priority,
                progress_status: 'open',
                risk_status:     'pending',
                completion:      0,
                dates:           $dates,
                assigned_to:     $assigned_to,
                created_by:      $created_by,
                party_id:        $party_id,
                parent_task_id:  $parent_task_id,
                level:           $level,
                norm_refs:       $norm_refs,
                tags:            $tags,
                external_ref:    $external_ref,
                created_at:      $now,
                updated_at:      $now,
                deleted_at:      NONE
            } RETURN AFTER",
        )
        .bind(("subject".to_owned(),       req.subject.clone()))
        .bind(("description".to_owned(),   req.description.unwrap_or_default()))
        .bind(("priority".to_owned(),      req.priority.as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "normal".into())))
        .bind(("dates".to_owned(),         dates_json))
        .bind(("assigned_to".to_owned(),   req.assigned_to.unwrap_or_default()))
        .bind(("created_by".to_owned(),    actor_id.to_owned()))
        .bind(("party_id".to_owned(),      req.party_id.unwrap_or_default()))
        .bind(("parent_task_id".to_owned(),req.parent_task_id.clone().unwrap_or_default()))
        .bind(("level".to_owned(),         req.parent_task_id.is_some() as u8))
        .bind(("norm_refs".to_owned(),     req.norm_refs.unwrap_or_default()))
        .bind(("tags".to_owned(),          req.tags.unwrap_or_default()))
        .bind(("external_ref".to_owned(),  ext_json))
        .bind(("now".to_owned(),           now.to_rfc3339()))
        .await
        .map_err(db_err)?;

    let task: Task = take_one(&mut res, 0)?
        .ok_or_else(|| AppError::Internal("CREATE returned no record".into()))?;

    write_outbox(
        db,
        topics::TASK_EVENTS.to_owned(),
        task.id.clone(),
        "task.created".to_owned(),
        &TaskEvent::Created(shared::events::TaskCreated {
            task_id:     task.id.clone(),
            subject:     task.subject.clone(),
            priority:    task.priority.clone(),
            assigned_to: task.assigned_to.clone(),
            party_id:    task.party_id.clone(),
            planned_end: task.dates.planned_end,
        }),
        actor_id.to_owned(),
        tenant_id,
    )
        .await?;

    Ok(task)
}

// ─────────────────────────────────────────────────────────────
// Update (partial)
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db, actor_id))]
pub async fn update_task(
    db:        &Surreal<Client>,
    id:        &str,
    req:       UpdateTaskRequest,
    actor_id:  &str,
    tenant_id: Uuid,
) -> Result<Task> {
    let mut set_fields: Vec<String> = vec!["updated_at = time::now()".to_owned()];

    if let Some(ref v) = req.subject         { set_fields.push(format!("subject = '{v}'")); }
    if let Some(ref v) = req.description     { set_fields.push(format!("description = '{v}'")); }
    if let Some(ref v) = req.priority        { set_fields.push(format!("priority = '{v}'")); }
    if let Some(ref v) = req.progress_status { set_fields.push(format!("progress_status = '{v}'")); }
    if let Some(ref v) = req.risk_status     { set_fields.push(format!("risk_status = '{v}'")); }
    if let Some(v)     = req.completion      { set_fields.push(format!("completion = {v}")); }
    if let Some(ref v) = req.assigned_to     { set_fields.push(format!("assigned_to = '{v}'")); }
    if let Some(ref v) = req.norm_refs       {
        let json = serde_json::to_string(v).map_err(ser_err)?;
        set_fields.push(format!("norm_refs = {json}"));
    }
    if let Some(ref v) = req.tags           {
        let json = serde_json::to_string(v).map_err(ser_err)?;
        set_fields.push(format!("tags = {json}"));
    }

    let set_clause = set_fields.join(", ");
    let query = format!(
        "UPDATE type::record('task', $id) SET {set_clause} RETURN AFTER"
    );

    let mut res = db
        .query(query)
        .bind(("id".to_owned(), id.to_owned()))
        .await
        .map_err(db_err)?;

    let task: Task = take_one(&mut res, 0)?
        .ok_or_else(|| AppError::NotFound(format!("task:{id}")))?;

    write_outbox(
        db,
        topics::TASK_EVENTS.to_owned(),
        task.id.clone(),
        "task.updated".to_owned(),
        &TaskEvent::Updated(shared::events::TaskUpdated {
            task_id:  task.id.clone(),
            subject:  None,
            priority: None,
        }),
        actor_id.to_owned(),
        tenant_id,
    )
        .await?;

    Ok(task)
}

// ─────────────────────────────────────────────────────────────
// Status transition
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db, actor_id))]
pub async fn transition_status(
    db:         &Surreal<Client>,
    id:         &str,
    new_status: ProgressStatus,
    actor_id:   &str,
    tenant_id:  Uuid,
) -> Result<Task> {
    let current = get_task(db, id).await?;

    if current.progress_status.is_terminal() ||
        !current.progress_status.can_transition_to(&new_status) {
        return Err(AppError::InvalidTransition {
            from: current.progress_status.to_string(),
            to:   new_status.to_string(),
        });
    }

    let completion = match new_status {
        ProgressStatus::Completed | ProgressStatus::Verified => 100,
        _ => current.completion,
    };

    let mut res = db
        .query(
            "UPDATE type::record('task', $id) MERGE {
                progress_status: $status,
                completion:      $completion,
                updated_at:      time::now()
            } RETURN AFTER",
        )
        .bind(("id".to_owned(),         id.to_owned()))
        .bind(("status".to_owned(),     new_status.to_string()))
        .bind(("completion".to_owned(), completion))
        .await
        .map_err(db_err)?;

    let task: Task = take_one(&mut res, 0)?
        .ok_or_else(|| AppError::NotFound(format!("task:{id}")))?;

    write_outbox(
        db,
        topics::TASK_EVENTS.to_owned(),
        task.id.clone(),
        "task.status_changed".to_owned(),
        &TaskEvent::StatusChanged(TaskStatusChanged {
            task_id:     task.id.clone(),
            from_status: current.progress_status,
            to_status:   new_status,
            completion,
        }),
        actor_id.to_owned(),
        tenant_id,
    )
        .await?;

    Ok(task)
}

// ─────────────────────────────────────────────────────────────
// Risk update
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db, actor_id))]
pub async fn update_risk(
    db:        &Surreal<Client>,
    id:        &str,
    new_risk:  RiskStatus,
    reason:    Option<String>,
    actor_id:  &str,
    tenant_id: Uuid,
) -> Result<Task> {
    let current = get_task(db, id).await?;

    let mut res = db
        .query(
            "UPDATE type::record('task', $id) MERGE {
                risk_status: $risk,
                updated_at:  time::now()
            } RETURN AFTER",
        )
        .bind(("id".to_owned(),   id.to_owned()))
        .bind(("risk".to_owned(), new_risk.to_string()))
        .await
        .map_err(db_err)?;

    let task: Task = take_one(&mut res, 0)?
        .ok_or_else(|| AppError::NotFound(format!("task:{id}")))?;

    write_outbox(
        db,
        topics::TASK_EVENTS.to_owned(),
        task.id.clone(),
        "task.risk_changed".to_owned(),
        &TaskEvent::RiskChanged(shared::events::TaskRiskChanged {
            task_id:   task.id.clone(),
            from_risk: current.risk_status,
            to_risk:   new_risk,
            reason,
        }),
        actor_id.to_owned(),
        tenant_id,
    )
        .await?;

    Ok(task)
}

// ─────────────────────────────────────────────────────────────
// Soft delete
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db))]
pub async fn soft_delete(
    db:        &Surreal<Client>,
    id:        &str,
    actor_id:  &str,
    tenant_id: Uuid,
) -> Result<()> {
    db.query(
        "UPDATE type::record('task', $id) SET
            deleted_at = time::now(),
            updated_at = time::now()",
    )
        .bind(("id".to_owned(), id.to_owned()))
        .await
        .map_err(db_err)?;

    write_outbox(
        db,
        topics::TASK_EVENTS.to_owned(),
        format!("task:{id}"),
        "task.deleted".to_owned(),
        &TaskEvent::Deleted(shared::events::TaskDeleted {
            task_id: format!("task:{id}"),
        }),
        actor_id.to_owned(),
        tenant_id,
    )
        .await?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────
// History
// ─────────────────────────────────────────────────────────────

#[instrument(skip(db))]
pub async fn get_history(db: &Surreal<Client>, task_id: &str) -> Result<Vec<TaskHistory>> {
    let mut res = db
        .query(
            "SELECT * FROM task_history
             WHERE task_id = type::record('task', $id)
             ORDER BY occurred_at ASC",
        )
        .bind(("id".to_owned(), task_id.to_owned()))
        .await
        .map_err(db_err)?;

    take_vec(&mut res, 0)
}

// ─────────────────────────────────────────────────────────────
// Outbox helper (Transactional Outbox Pattern)
// ─────────────────────────────────────────────────────────────

async fn write_outbox(
    db:         &Surreal<Client>,
    topic:      String,
    aggregate:  String,
    event_type: String,
    payload:    &impl serde::Serialize,
    _actor_id:   String,
    tenant_id:  Uuid,
) -> Result<()> {
    let payload_json = serde_json::to_string(payload).map_err(ser_err)?;

    db.query(
        "CREATE outbox SET
            topic       = $topic,
            aggregate   = $aggregate,
            event_type  = $event_type,
            payload     = $payload,
            tenant_id   = $tenant_id,
            created_at  = time::now(),
            sent_at     = NONE,
            failed_at   = NONE,
            retry_count = 0",
    )
        .bind(("topic".to_owned(),      topic))
        .bind(("aggregate".to_owned(),  aggregate))
        .bind(("event_type".to_owned(), event_type))
        .bind(("payload".to_owned(),    payload_json))
        .bind(("tenant_id".to_owned(),  tenant_id.to_string()))
        .await
        .map_err(db_err)?;

    Ok(())
}
