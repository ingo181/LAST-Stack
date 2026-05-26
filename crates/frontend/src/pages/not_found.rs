// ─────────────────────────────────────────────────────────────
// crates/frontend/src/pages/not_found.rs
// ─────────────────────────────────────────────────────────────
use leptos::prelude::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="empty-state">
            <h2>"404 — Page not found"</h2>
            <p><a href="/">"← Back to Tasks"</a></p>
        </div>
    }
}
