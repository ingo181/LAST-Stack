// Single source of truth for all domain types.
// Used by: apqp-service (Axum), frontend (Leptos/WASM), notify-service.
// No framework-specific code here — pure Rust + serde + chrono.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// ─────────────────────────────────────────────────────────────
// Re-exports so consumers only need `use shared::models::*;`
// ─────────────────────────────────────────────────────────────
pub use self::api::{ApiResponse, PagedResponse, Pagination};
pub use self::contact::{Contact, ContactStatus, ContactSummary};
pub use self::party::{Address, Party, PartyKind};
pub use self::task::{
    CreateTaskRequest, ExternalRef, Priority, ProgressStatus,
    RiskStatus, Task, TaskDates, TaskHistory, TaskSummary,
    UpdateTaskRequest,
};
pub use self::user::{User, UserRole};

// ─────────────────────────────────────────────────────────────
// 1.  API wrapper types
// ─────────────────────────────────────────────────────────────
pub mod api {
    use serde::{Deserialize, Serialize};

    /// Standard JSON envelope for every API response.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApiResponse<T> {
        pub data:    T,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub message: Option<String>,
    }

    impl<T> ApiResponse<T> {
        pub fn ok(data: T) -> Self {
            Self { data, message: None }
        }
        pub fn with_message(data: T, msg: impl Into<String>) -> Self {
            Self { data, message: Some(msg.into()) }
        }
    }

    /// Paginated list response.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PagedResponse<T> {
        pub items:      Vec<T>,
        pub total:      u64,
        pub page:       u32,
        pub page_size:  u32,
    }

    /// Query parameters for list endpoints.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Pagination {
        #[serde(default = "default_page")]
        pub page:      u32,
        #[serde(default = "default_page_size")]
        pub page_size: u32,
    }

    fn default_page()      -> u32 { 1 }
    fn default_page_size() -> u32 { 25 }
}

// ─────────────────────────────────────────────────────────────
// 2.  User  (mirrors JWT sub + Authentik claims)
// ─────────────────────────────────────────────────────────────
pub mod user {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct User {
        pub id:           String,           // SurrealDB record id
        pub email:        String,
        pub display_name: String,
        pub role:         UserRole,
        pub locale:       String,
        pub active:       bool,
        pub created_at:   DateTime<Utc>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum UserRole {
        Admin,
        Manager,
        Member,
        Guest,
    }

    impl fmt::Display for UserRole {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                Self::Admin   => "admin",
                Self::Manager => "manager",
                Self::Member  => "member",
                Self::Guest   => "guest",
            };
            write!(f, "{s}")
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 3.  Party  (VDA QDX: BuyerParty, SellerParty, Manufacturer…)
// ─────────────────────────────────────────────────────────────
pub mod party {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Party {
        pub id:          String,
        pub party_id:    String,            // Internal business ID (unique)
        pub name:        Option<String>,
        pub kind:        PartyKind,
        pub address:     Option<Address>,
        pub vda_duns:    Option<String>,    // DUNS for VDA QDX exchange
        pub created_at:  DateTime<Utc>,
        pub updated_at:  DateTime<Utc>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum PartyKind {
        Customer,
        Supplier,
        Internal,
        Other,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Address {
        pub street:  Option<String>,
        pub city:    Option<String>,
        pub zip:     Option<String>,
        pub state:   Option<String>,
        pub country: Option<String>,  // ISO 3166-1 alpha-2
    }
}

// ─────────────────────────────────────────────────────────────
// 4.  Contact  (Responsible — person assigned to tasks)
// ─────────────────────────────────────────────────────────────
pub mod contact {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Contact {
        pub id:           String,
        pub first_name:   String,
        pub last_name:    String,
        pub email:        Option<String>,
        pub phone:        Option<String>,
        pub role:         String,
        pub department:   String,
        pub party_id:     Option<String>,   // → Party record
        pub status:       ContactStatus,
        pub avatar_url:   Option<String>,
        pub assigned_to:  Option<String>,   // → User record
        pub address:      Option<party::Address>,
        pub created_at:   DateTime<Utc>,
        pub updated_at:   DateTime<Utc>,
    }

    impl Contact {
        pub fn full_name(&self) -> String {
            format!("{} {}", self.first_name, self.last_name)
        }
    }

    /// Lightweight projection for list views / dropdowns.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ContactSummary {
        pub id:         String,
        pub full_name:  String,
        pub role:       String,
        pub party_name: Option<String>,
        pub status:     ContactStatus,
        pub avatar_url: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ContactStatus {
        Active,
        Inactive,
        Prospect,
    }

    impl fmt::Display for ContactStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Active   => write!(f, "active"),
                Self::Inactive => write!(f, "inactive"),
                Self::Prospect => write!(f, "prospect"),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 5.  Task  (Herzstück / core entity)
//     Two separate status models per VDA QDX 3.0 / Fachkonzept:
//       • ProgressStatus  — internal workflow
//       • RiskStatus      — GYR traffic light for customer reports
// ─────────────────────────────────────────────────────────────
pub mod task {
    use super::*;

    // ── Full Task record (returned from DB) ───────────────────

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Task {
        pub id:              String,
        pub subject:         String,
        pub description:     String,
        pub priority:        Priority,

        // Two independent status models (Fachkonzept §3.3.2)
        pub progress_status: ProgressStatus,
        pub risk_status:     RiskStatus,

        /// 0–100 — completion degree
        pub completion:      u8,

        // VDA QDX date model (Fachkonzept §3.3.1)
        pub dates:           TaskDates,

        // Relations (stored as SurrealDB record IDs)
        pub assigned_to:     Option<String>,   // → Contact
        pub created_by:      String,           // → User
        pub party_id:        Option<String>,   // → Party
        pub parent_task_id:  Option<String>,   // → Task (subtask)
        pub level:           u8,               // 0–2, max 3 levels (QDX §4.3)

        // ISO norm references, e.g. ["ISO 9001:2015 §10.2"]
        pub norm_refs:       Vec<String>,

        // Tags for filtering
        pub tags:            Vec<String>,

        // Optional external system link (Jira, OpenProject, …)
        pub external_ref:    Option<ExternalRef>,

        pub created_at:      DateTime<Utc>,
        pub updated_at:      DateTime<Utc>,
        pub deleted_at:      Option<DateTime<Utc>>,
    }

    /// VDA QDX 3.0 — four date pairs (planned / estimated / finalized / agreed)
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    #[serde(default)]
    pub struct TaskDates {
        /// Original plan — must not change after project kick-off
        pub planned_start:   Option<DateTime<Utc>>,
        pub planned_end:     Option<DateTime<Utc>>,

        /// Rolling forecast — updated each status cycle
        pub estimated_start: Option<DateTime<Utc>>,
        pub estimated_end:   Option<DateTime<Utc>>,

        /// Actuals — set when work begins / completes
        pub finalized_start: Option<DateTime<Utc>>,
        pub finalized_end:   Option<DateTime<Utc>>,

        /// Bilaterally agreed date after escalation
        pub agreed_date:     Option<DateTime<Utc>>,
    }

    /// Link to an external task tracker without schema changes.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct ExternalRef {
        pub system:    String,           // "jira" | "openproject" | "plane"
        pub issue_key: String,           // "PROJ-123"
        pub url:       Option<String>,
        pub synced_at: Option<DateTime<Utc>>,
    }

    /// Lightweight projection for list / table views.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct TaskSummary {
        pub id:              String,
        pub subject:         String,
        pub priority:        Priority,
        pub progress_status: ProgressStatus,
        pub risk_status:     RiskStatus,
        pub completion:      u8,
        pub planned_end:     Option<DateTime<Utc>>,
        pub assigned_to:     Option<String>,
        pub party_id:        Option<String>,
    }

    /// Append-only history entry written by SurrealDB Events.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct TaskHistory {
        pub id:          String,
        pub task_id:     String,
        pub field:       String,         // "progress_status" | "risk_status"
        pub old_value:   String,
        pub new_value:   String,
        pub changed_by:  String,         // User ID from TenantContext
        pub occurred_at: DateTime<Utc>,
    }

    // ── Enumerations ──────────────────────────────────────────

    /// Internal workflow status (Fachkonzept §3.3.2)
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ProgressStatus {
        Open,
        InProgress,
        WaitingForFeedback,
        Completed,
        Verified,           // effectiveness confirmed (8D D7) — terminal
        Rejected,
        Deferred,
        Cancelled,          // terminal
    }

    impl ProgressStatus {
        /// Returns true if no further transitions are allowed.
        pub fn is_terminal(&self) -> bool {
            matches!(self, Self::Verified | Self::Cancelled)
        }

        /// Allowed successor states (mirrors allowed_task_transitions seed data).
        pub fn allowed_next(&self) -> &'static [ProgressStatus] {
            use ProgressStatus::*;
            match self {
                Open               => &[InProgress, Deferred, Cancelled],
                InProgress         => &[WaitingForFeedback, Completed, Deferred, Cancelled],
                WaitingForFeedback => &[InProgress, Completed, Cancelled],
                Completed          => &[Verified, Rejected],
                Rejected           => &[Open, InProgress],
                Deferred           => &[Open, Cancelled],
                Verified           => &[],
                Cancelled          => &[],
            }
        }

        pub fn can_transition_to(&self, next: &ProgressStatus) -> bool {
            self.allowed_next().contains(next)
        }
    }


    impl std::str::FromStr for ProgressStatus {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "open"                 => Ok(Self::Open),
                "in_progress"          => Ok(Self::InProgress),
                "waiting_for_feedback" => Ok(Self::WaitingForFeedback),
                "completed"            => Ok(Self::Completed),
                "verified"             => Ok(Self::Verified),
                "rejected"             => Ok(Self::Rejected),
                "deferred"             => Ok(Self::Deferred),
                "cancelled"            => Ok(Self::Cancelled),
                other => Err(format!("unknown progress_status: {other}")),
            }
        }
    }
    impl fmt::Display for ProgressStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                Self::Open               => "open",
                Self::InProgress         => "in_progress",
                Self::WaitingForFeedback => "waiting_for_feedback",
                Self::Completed          => "completed",
                Self::Verified           => "verified",
                Self::Rejected           => "rejected",
                Self::Deferred           => "deferred",
                Self::Cancelled          => "cancelled",
            };
            write!(f, "{s}")
        }
    }

    /// GYR traffic-light risk status for customer status reports (VDA Band 7)
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum RiskStatus {
        /// All targets met — no or negligible deviation
        Green,
        /// Substantial deviation — corrective measures defined, target still reachable
        Yellow,
        /// Serious deviation — SOP/project end at risk
        YellowRed,
        /// Target not achievable — No Go
        Red,
        /// No assessment yet
        Pending,
        NotApplicable,
    }


    impl std::str::FromStr for RiskStatus {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "green"          => Ok(Self::Green),
                "yellow"         => Ok(Self::Yellow),
                "yellow_red"     => Ok(Self::YellowRed),
                "red"            => Ok(Self::Red),
                "pending"        => Ok(Self::Pending),
                "not_applicable" => Ok(Self::NotApplicable),
                other => Err(format!("unknown risk_status: {other}")),
            }
        }
    }
    impl fmt::Display for RiskStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                Self::Green         => "green",
                Self::Yellow        => "yellow",
                Self::YellowRed     => "yellow_red",
                Self::Red           => "red",
                Self::Pending       => "pending",
                Self::NotApplicable => "not_applicable",
            };
            write!(f, "{s}")
        }
    }

    /// Task priority
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Priority {
        High,
        Normal,
        Low,
    }

    impl fmt::Display for Priority {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::High   => write!(f, "high"),
                Self::Normal => write!(f, "normal"),
                Self::Low    => write!(f, "low"),
            }
        }
    }

    impl std::str::FromStr for Priority {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "high"   => Ok(Self::High),
                "normal" => Ok(Self::Normal),
                "low"    => Ok(Self::Low),
                other    => Err(format!("unknown priority: {other}")),
            }
        }
    }

    // ── Request / Response DTOs ───────────────────────────────

    /// POST /tasks
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateTaskRequest {
        pub subject:         String,
        pub description:     Option<String>,
        pub priority:        Option<Priority>,
        pub dates:           Option<TaskDates>,
        pub assigned_to:     Option<String>,
        pub party_id:        Option<String>,
        pub parent_task_id:  Option<String>,
        pub norm_refs:       Option<Vec<String>>,
        pub tags:            Option<Vec<String>>,
        pub external_ref:    Option<ExternalRef>,
    }

    /// PATCH /tasks/:id  — all fields optional (partial update)
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct UpdateTaskRequest {
        pub subject:         Option<String>,
        pub description:     Option<String>,
        pub priority:        Option<Priority>,
        pub progress_status: Option<ProgressStatus>,
        pub risk_status:     Option<RiskStatus>,
        pub completion:      Option<u8>,
        pub dates:           Option<TaskDates>,
        pub assigned_to:     Option<String>,
        pub norm_refs:       Option<Vec<String>>,
        pub tags:            Option<Vec<String>>,
        pub external_ref:    Option<ExternalRef>,
    }
}