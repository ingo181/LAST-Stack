// ─────────────────────────────────────────────────────────────
// crates/frontend/src/components/task_form.rs
// Thaw Dialog + Form — neuen Task anlegen
// ─────────────────────────────────────────────────────────────
use leptos::prelude::*;
use thaw::{
    Button, ButtonAppearance, Dialog, DialogBody, DialogSurface,
    DialogTitle, DialogActions, Field, Input, Select,
    Textarea,
};
use shared::models::task::{CreateTaskRequest, Priority};

#[component]
pub fn TaskFormDialog(
    open:      RwSignal<bool>,
    on_submit: Callback<CreateTaskRequest>,
) -> impl IntoView {
    let subject     = RwSignal::new(String::new());
    let priority    = RwSignal::new(String::from("normal"));
    let assigned_to = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let error_msg   = RwSignal::new(Option::<String>::None);

    let on_save = move |_| {
        let s = subject.get();
        if s.trim().is_empty() {
            error_msg.set(Some("Subject is required.".into()));
            return;
        }
        error_msg.set(None);

        let priority_val = match priority.get().as_str() {
            "high" => Priority::High,
            "low"  => Priority::Low,
            _      => Priority::Normal,
        };

        on_submit.run(CreateTaskRequest {
            subject:        s,
            description:    Some(description.get()).filter(|s| !s.is_empty()),
            priority:       Some(priority_val),
            assigned_to:    Some(assigned_to.get()).filter(|s| !s.is_empty()),
            dates:          None,
            party_id:       None,
            parent_task_id: None,
            norm_refs:      None,
            tags:           None,
            external_ref:   None,
        });

        // Reset form
        subject.set(String::new());
        priority.set("normal".into());
        assigned_to.set(String::new());
        description.set(String::new());
        open.set(false);
    };

    let on_cancel = move |_| {
        error_msg.set(None);
        open.set(false);
    };

    view! {
        <Dialog open>
            <DialogSurface>
                <DialogTitle>"New Task"</DialogTitle>
                <DialogBody>
                    // Validation error
                    {move || error_msg.get().map(|e| view! {
                        <div class="error-banner">{e}</div>
                    })}

                    <Field label="Subject" required=true>
                        <Input
                            value=subject
                            placeholder="Describe the task…"
                        />
                    </Field>

                    <Field label="Priority">
                        <Select value=priority>
                            <option value="high">"High"</option>
                            <option value="normal">"Normal"</option>
                            <option value="low">"Low"</option>
                        </Select>
                    </Field>

                    <Field label="Assigned To">
                        <Input
                            value=assigned_to
                            placeholder="Contact name…"
                        />
                    </Field>

                    <Field label="Description">
                        <Textarea
                            value=description
                            placeholder="Additional details…"
                        />
                    </Field>
                </DialogBody>
                <DialogActions>
                    <Button on_click=on_cancel>"Cancel"</Button>
                    <Button appearance=ButtonAppearance::Primary on_click=on_save>
                        "Save"
                    </Button>
                </DialogActions>
            </DialogSurface>
        </Dialog>
    }
}
