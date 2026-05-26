// shared/src/events.rs
//
// All Kafka event types used across services.
// Every event is wrapped in EventEnvelope<T> which carries
// routing metadata (tenant_id, event_type, occurred_at).
//
// Pattern: Transactional Outbox
//   apqp-service writes to SurrealDB `outbox` table atomically
//   with the business write. A background worker reads unsent
//   rows and publishes them here, then marks `sent_at`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::task::{Priority, ProgressStatus, RiskStatus};

// ─────────────────────────────────────────────────────────────
// Kafka topic constants
// ─────────────────────────────────────────────────────────────
pub mod topics {
    /// apqp-service → all consumers
    pub const TASK_EVENTS:       &str = "task.events";
    /// apqp-service → notify-service, external connectors
    pub const PROJECT_EVENTS:    &str = "project.events";
    /// complaint-service → notify-service, external connectors
    pub const COMPLAINT_EVENTS:  &str = "complaint.events";
    /// all services → notify-service (email, WebSocket, Slack)
    pub const NOTIFICATIONS:     &str = "notifications";
    /// all services → monitoring / dead-letter handler
    pub const DEAD_LETTER:       &str = "dead.letter";
}

// ─────────────────────────────────────────────────────────────
// Generic event envelope
// ─────────────────────────────────────────────────────────────

/// Wraps every domain event published to Kafka.
/// The `payload` field carries the type-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    /// Globally unique event ID (UUID v4)
    pub event_id:    Uuid,
    /// Dot-separated event type, e.g. "task.created"
    pub event_type:  String,
    /// Aggregate root ID, e.g. "task:t1" (SurrealDB record id)
    pub aggregate:   String,
    /// Tenant for multi-tenancy routing and DB selection
    pub tenant_id:   Uuid,
    /// User who triggered the event (from TenantContext / JWT sub)
    pub actor_id:    String,
    /// Business event timestamp (not the Kafka ingestion time)
    pub occurred_at: DateTime<Utc>,
    /// Type-specific payload
    pub payload:     T,
}

impl<T: Serialize + for<'de> Deserialize<'de>> EventEnvelope<T> {
    pub fn new(
        event_type: impl Into<String>,
        aggregate:  impl Into<String>,
        tenant_id:  Uuid,
        actor_id:   impl Into<String>,
        payload:    T,
    ) -> Self {
        Self {
            event_id:    Uuid::new_v4(),
            event_type:  event_type.into(),
            aggregate:   aggregate.into(),
            tenant_id,
            actor_id:    actor_id.into(),
            occurred_at: Utc::now(),
            payload,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Task events  (topic: task.events)
// ─────────────────────────────────────────────────────────────

/// Discriminated union of all task event payloads.
/// Published by apqp-service, consumed by notify-service,
/// audit-service, and external connectors (Jira, OpenProject).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskEvent {
    Created(TaskCreated),
    Updated(TaskUpdated),
    StatusChanged(TaskStatusChanged),
    RiskChanged(TaskRiskChanged),
    Completed(TaskCompleted),
    Cancelled(TaskCancelled),
    Deleted(TaskDeleted),
    AssigneeChanged(TaskAssigneeChanged),
}

impl TaskEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Created(_)         => "task.created",
            Self::Updated(_)         => "task.updated",
            Self::StatusChanged(_)   => "task.status_changed",
            Self::RiskChanged(_)     => "task.risk_changed",
            Self::Completed(_)       => "task.completed",
            Self::Cancelled(_)       => "task.cancelled",
            Self::Deleted(_)         => "task.deleted",
            Self::AssigneeChanged(_) => "task.assignee_changed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreated {
    pub task_id:    String,
    pub subject:    String,
    pub priority:   Priority,
    pub assigned_to: Option<String>,
    pub party_id:   Option<String>,
    pub planned_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdated {
    pub task_id:  String,
    pub subject:  Option<String>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusChanged {
    pub task_id:    String,
    pub from_status: ProgressStatus,
    pub to_status:   ProgressStatus,
    pub completion:  u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRiskChanged {
    pub task_id:     String,
    pub from_risk:   RiskStatus,
    pub to_risk:     RiskStatus,
    pub reason:      Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleted {
    pub task_id:      String,
    pub completed_at: DateTime<Utc>,
    pub completion:   u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCancelled {
    pub task_id: String,
    pub reason:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDeleted {
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssigneeChanged {
    pub task_id:      String,
    pub from_assignee: Option<String>,
    pub to_assignee:   Option<String>,
}

// ─────────────────────────────────────────────────────────────
// Notification events  (topic: notifications)
// ─────────────────────────────────────────────────────────────

/// Consumed by notify-service to dispatch email / WebSocket / Slack.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NotificationEvent {
    TaskOverdue(TaskOverdueNotification),
    TaskAssigned(TaskAssignedNotification),
    StatusChanged(StatusChangedNotification),
    RiskEscalated(RiskEscalatedNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOverdueNotification {
    pub task_id:      String,
    pub subject:      String,
    pub assignee_id:  String,
    pub planned_end:  DateTime<Utc>,
    pub days_overdue: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignedNotification {
    pub task_id:     String,
    pub subject:     String,
    pub assignee_id: String,
    pub assigned_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusChangedNotification {
    pub task_id:    String,
    pub subject:    String,
    pub from_status: ProgressStatus,
    pub to_status:   ProgressStatus,
    pub actor_id:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEscalatedNotification {
    pub task_id:   String,
    pub subject:   String,
    pub risk:      RiskStatus,
    pub party_id:  Option<String>,
    pub actor_id:  String,
}

// ─────────────────────────────────────────────────────────────
// Dead-letter envelope  (topic: dead.letter)
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEvent {
    /// Original Kafka topic
    pub source_topic:  String,
    /// Original event_id
    pub source_event:  Uuid,
    /// Serialised original payload (JSON string)
    pub original_body: String,
    /// Error message from the failing consumer
    pub error:         String,
    pub failed_at:     DateTime<Utc>,
    pub retry_count:   u32,
}

// ─────────────────────────────────────────────────────────────
// Outbox row  (written to SurrealDB, read by outbox worker)
// ─────────────────────────────────────────────────────────────

/// In-memory representation of a row in the `outbox` SurrealDB table.
/// The worker deserialises this, publishes to Kafka, then sets `sent_at`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxRow {
    pub id:          String,            // SurrealDB record id
    pub topic:       String,
    pub aggregate:   String,
    pub event_type:  String,
    pub payload:     String,            // JSON-serialised EventEnvelope<T>
    pub tenant_id:   String,
    pub created_at:  DateTime<Utc>,
    pub sent_at:     Option<DateTime<Utc>>,
    pub failed_at:   Option<DateTime<Utc>>,
    pub retry_count: u32,
}