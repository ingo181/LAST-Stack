// ─────────────────────────────────────────────────────────────
// crates/frontend/src/pages/task_list.rs
// Route: / und /tasks
// ─────────────────────────────────────────────────────────────
use leptos::prelude::*;
use reactive_graph::owner::LocalStorage;
use shared::models::{api::PagedResponse, task::{CreateTaskRequest, TaskSummary}};
use crate::{
    api,
    components::{
        task_form::TaskFormDialog,
        task_table::TaskTable,
    },
};

#[component]
pub fn TaskListPage() -> impl IntoView {
    // ── State ─────────────────────────────────────────────────
    let tasks    = RwSignal::new(Option::<PagedResponse<TaskSummary>>::None);
    let loading  = RwSignal::new(false);
    let error    = RwSignal::new(Option::<String>::None);
    let form_open = RwSignal::new(false);
    let page     = RwSignal::new(1u32);

    // ── Fetch action ─────────────────────────────────────────
    let fetch: Action<u32, (), LocalStorage> = Action::new_unsync(move |p: &u32| {
        let p = *p;
        async move {
            loading.set(true);
            error.set(None);
            match api::list_tasks(None, p).await {
                Ok(data) => tasks.set(Some(data)),
                Err(e)   => error.set(Some(e)),
            }
            loading.set(false);
        }
    });

    // Initial load
    fetch.dispatch(1);

    let on_reload = Callback::new(move |_: ()| {
        fetch.dispatch(page.get_untracked());
    });

    // ── Create action ────────────────────────────────────────
    let create: Action<CreateTaskRequest, (), LocalStorage> = Action::new_unsync(move |req: &CreateTaskRequest| {
        let req = req.clone();
        async move {
            match api::create_task(req).await {
                Ok(_)  => { fetch.dispatch(page.get_untracked()); }
                Err(e) => { error.set(Some(e)); }
            }
        }
    });

    let on_submit = Callback::new(move |req: CreateTaskRequest| {
        create.dispatch(req);
    });

    // ── View ─────────────────────────────────────────────────
    view! {
        <div class="page">
            <div class="page-header">
                <h1 class="page-title">"Tasks"</h1>
            </div>

            <TaskTable
                tasks=tasks.read_only()
                loading=loading.read_only()
                error=error.read_only()
                on_add=Callback::new(move |_| form_open.set(true))
                on_reload
            />

            <TaskFormDialog
                open=form_open
                on_submit
            />
        </div>
    }
}
