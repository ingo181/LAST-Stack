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

    let edit_task_id = RwSignal::new(Option::<String>::None);
    let edit_task_data = RwSignal::new(Option::<shared::models::task::Task>::None);
    let edit_form_open = RwSignal::new(false);

    let load_edit: Action<String, (), LocalStorage> = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        async move {
            match crate::api::get_task(&id).await {
                Ok(task) => {
                    edit_task_id.set(Some(task.id.clone()));
                    edit_task_data.set(Some(task));
                    edit_form_open.set(true);
                }
                Err(e) => { error.set(Some(e)); }
            }
        }
    });

    let on_edit = Callback::new(move |id: String| {
        load_edit.dispatch(id);
    });

    let delete_action: Action<String, (), LocalStorage> = Action::new_unsync(move |id: &String| {
        let id = id.clone();
        async move {
            match crate::api::delete_task(&id).await {
                Ok(_)  => { fetch.dispatch(page.get_untracked()); }
                Err(e) => { error.set(Some(e)); }
            }
        }
    });

    let on_delete = Callback::new(move |id: String| {
        delete_action.dispatch(id);
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
                on_edit
                on_delete
                on_reload
            />

            <TaskFormDialog
                open=form_open
                on_submit
            />

            // Edit Dialog
            {move || edit_task_data.get().map(|task| view! {
                <TaskFormDialog
                    open=edit_form_open
                    title="Edit Task"
                    initial_subject=task.subject.clone()
                    initial_company=task.party_id.clone().unwrap_or_default()
                    initial_assigned_to=task.assigned_to.clone().unwrap_or_default()
                    initial_priority=task.priority.to_string()
                    initial_description=task.description.clone()
                    on_submit=Callback::new(move |req: CreateTaskRequest| {
                        if let Some(id) = edit_task_id.get_untracked() {
                            leptos::task::spawn_local(async move {
                                match crate::api::update_task(&id, req).await {
                                    Ok(_)  => { fetch.dispatch(page.get_untracked()); }
                                    Err(e) => { error.set(Some(e)); }
                                }
                            });
                        }
                        edit_task_id.set(None);
                        edit_task_data.set(None);
                        edit_form_open.set(false);
                    })
                />
            })}
        </div>
    }
}
