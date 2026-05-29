// crates/frontend/src/components/task_form.rs
use leptos::prelude::*;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogBody, DialogSurface,
    DialogTitle, DialogActions, Field, Input, Select, Textarea,
};
use shared::models::task::{CreateTaskRequest, Priority};

#[component]
pub fn TaskFormDialog(
    open:      RwSignal<bool>,
    on_submit: Callback<CreateTaskRequest>,
) -> impl IntoView {
    let subject      = RwSignal::new(String::new());
    let company      = RwSignal::new(String::new());
    let assigned_to  = RwSignal::new(String::new());
    let priority     = RwSignal::new(String::from("low"));
    let status       = RwSignal::new(String::from("open"));
    let start_date   = RwSignal::new(String::new());
    let due_date_val = RwSignal::new(String::new());
    let details      = RwSignal::new(String::new());
    let error_msg    = RwSignal::new(Option::<String>::None);

    let reset = move || {
        subject.set(String::new());
        company.set(String::new());
        assigned_to.set(String::new());
        priority.set("low".into());
        status.set("open".into());
        start_date.set(String::new());
        due_date_val.set(String::new());
        details.set(String::new());
        error_msg.set(None);
    };

    let on_save = move |_| {
        let s = subject.get();
        if s.trim().is_empty() {
            error_msg.set(Some("Subject is required.".into()));
            return;
        }
        error_msg.set(None);
        let priority_val = match priority.get().as_str() {
            "high"   => Priority::High,
            "normal" => Priority::Normal,
            _        => Priority::Low,
        };
        on_submit.run(CreateTaskRequest {
            subject:        s,
            description:    Some(details.get()).filter(|s| !s.is_empty()),
            priority:       Some(priority_val),
            assigned_to:    Some(assigned_to.get()).filter(|s| !s.is_empty()),
            dates:          None,
            party_id:       Some(company.get()).filter(|s| !s.is_empty()),
            parent_task_id: None,
            norm_refs:      None,
            tags:           None,
            external_ref:   None,
        });
        reset();
        open.set(false);
    };

    let on_cancel = move |_| { reset(); open.set(false); };

    view! {
        <Dialog open>
            <DialogSurface>
                <DialogTitle>"New Task"</DialogTitle>
                <DialogBody>
                    <div class="task-form">
                        {move || error_msg.get().map(|e| view! {
                            <div class="error-banner">{e}</div>
                        })}
                        <div class="form-row">
                            <Field label="Subject" required=true>
                                <Input value=subject placeholder="Task description…" />
                            </Field>
                        </div>
                        <div class="form-row form-row-2col">
                            <Field label="Company">
                                <Input value=company placeholder="Company name…" />
                            </Field>
                            <Field label="Assigned To">
                                <Input value=assigned_to placeholder="Responsible person…" />
                            </Field>
                        </div>
                        <div class="form-row form-row-2col">
                            <Field label="Priority">
                                <div class="priority-select-wrap">
                                    <span class=move || format!("priority-indicator priority-{}", priority.get()) />
                                    <Select value=priority>
                                        <option value="high">"High"</option>
                                        <option value="normal">"Normal"</option>
                                        <option value="low">"Low"</option>
                                    </Select>
                                </div>
                            </Field>
                            <Field label="Status">
                                <Select value=status>
                                    <option value="open">"Open"</option>
                                    <option value="in_progress">"In Progress"</option>
                                    <option value="completed">"Completed"</option>
                                </Select>
                            </Field>
                        </div>
                        <div class="form-row form-row-2col">
                            <Field label="Start Date">
                                <input type="date" class="date-input"
                                    on:change=move |e| {
                                        use wasm_bindgen::JsCast;
                                        let val = e.target()
                                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                            .map(|el| el.value())
                                            .unwrap_or_default();
                                        start_date.set(val);
                                    }
                                />
                            </Field>
                            <Field label="Due Date">
                                <input type="date" class="date-input"
                                    on:change=move |e| {
                                        use wasm_bindgen::JsCast;
                                        let val = e.target()
                                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                            .map(|el| el.value())
                                            .unwrap_or_default();
                                        due_date_val.set(val);
                                    }
                                />
                            </Field>
                        </div>
                        <div class="form-row">
                            <Field label="Details">
                                <Textarea value=details placeholder="Additional information…" />
                            </Field>
                        </div>
                    </div>
                </DialogBody>
                <DialogActions>
                    <Button on_click=on_cancel>"Cancel"</Button>
                    <Button appearance=ButtonAppearance::Primary on_click=on_save>"Save"</Button>
                </DialogActions>
            </DialogSurface>
        </Dialog>
    }
}
