// ─────────────────────────────────────────────────────────────
// crates/frontend/src/components/status_badge.rs
// ─────────────────────────────────────────────────────────────
use leptos::prelude::*;
use shared::models::task::{Priority, ProgressStatus};

#[component]
pub fn ProgressBadge(status: ProgressStatus) -> impl IntoView {
    let (label, class) = match status {
        ProgressStatus::Open               => ("Open",               "badge badge-open"),
        ProgressStatus::InProgress         => ("In Progress",        "badge badge-in-progress"),
        ProgressStatus::WaitingForFeedback => ("Waiting",            "badge badge-in-progress"),
        ProgressStatus::Completed          => ("Completed",          "badge badge-completed"),
        ProgressStatus::Verified           => ("Verified",           "badge badge-completed"),
        ProgressStatus::Rejected           => ("Rejected",           "badge badge-high"),
        ProgressStatus::Deferred           => ("Deferred",           "badge badge-cancelled"),
        ProgressStatus::Cancelled          => ("Cancelled",          "badge badge-cancelled"),
    };
    view! { <span class=class>{label}</span> }
}

#[component]
pub fn PriorityBadge(priority: Priority) -> impl IntoView {
    let (label, class) = match priority {
        Priority::High   => ("High",   "badge badge-high"),
        Priority::Normal => ("Normal", "badge badge-normal"),
        Priority::Low    => ("Low",    "badge badge-low"),
    };
    view! { <span class=class>{label}</span> }
}
