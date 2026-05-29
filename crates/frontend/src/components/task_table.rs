use leptos::prelude::*;
use thaw::{Button, ButtonAppearance, ButtonSize, Spinner, Table, TableBody, TableCell,
           TableCellLayout, TableHeader, TableHeaderCell, TableRow};
use shared::models::{api::PagedResponse, task::TaskSummary};
use crate::components::status_badge::{PriorityBadge, ProgressBadge};

#[component]
pub fn TaskTable(
    tasks:     ReadSignal<Option<PagedResponse<TaskSummary>>>,
    loading:   ReadSignal<bool>,
    error:     ReadSignal<Option<String>>,
    on_add:    Callback<()>,
    on_reload: Callback<()>,
    on_edit:   Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    view! {
        <div>
            <div class="task-toolbar">
                <Button appearance=ButtonAppearance::Primary on_click=move |_| on_add.run(())>
                    "+ New Task"
                </Button>
                <Button on_click=move |_| on_reload.run(())>
                    "↻ Refresh"
                </Button>
            </div>

            {move || error.get().map(|e| view! {
                <div class="error-banner">{e}</div>
            })}

            {move || loading.get().then(|| view! {
                <div style="padding:40px;text-align:center;"><Spinner /></div>
            })}

            {move || tasks.get().map(|page| {
                if page.items.is_empty() {
                    view! {
                        <div class="empty-state">
                            <p>"No tasks yet."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHeaderCell>"Subject"</TableHeaderCell>
                                    <TableHeaderCell>"Priority"</TableHeaderCell>
                                    <TableHeaderCell>"Status"</TableHeaderCell>
                                    <TableHeaderCell>"Due"</TableHeaderCell>
                                    <TableHeaderCell>"Assigned To"</TableHeaderCell>
                                    <TableHeaderCell>"Progress"</TableHeaderCell>
                                    <TableHeaderCell>""</TableHeaderCell>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                <For
                                    each=move || page.items.clone()
                                    key=|t| t.id.clone()
                                    children=move |task| {
                                        let id_edit   = task.id.clone();
                                        let id_delete = task.id.clone();
                                        view! {
                                            <TableRow>
                                                <TableCell>
                                                    <TableCellLayout>
                                                        {task.subject.clone()}
                                                    </TableCellLayout>
                                                </TableCell>
                                                <TableCell>
                                                    <PriorityBadge priority=task.priority.clone() />
                                                </TableCell>
                                                <TableCell>
                                                    <ProgressBadge status=task.progress_status.clone() />
                                                </TableCell>
                                                <TableCell>
                                                    {task.planned_end
                                                        .map(|d| d.format("%d.%m.%Y").to_string())
                                                        .unwrap_or_else(|| "—".into())}
                                                </TableCell>
                                                <TableCell>
                                                    {task.assigned_to.clone()
                                                        .unwrap_or_else(|| "—".into())}
                                                </TableCell>
                                                <TableCell>
                                                    {format!("{}%", task.completion)}
                                                </TableCell>
                                                <TableCell>
                                                    <div class="action-buttons">
                                                        <Button
                                                            size=ButtonSize::Small
                                                            on_click=move |_| on_edit.run(id_edit.clone())
                                                        >"✏️"</Button>
                                                        <Button
                                                            size=ButtonSize::Small
                                                            on_click=move |_| on_delete.run(id_delete.clone())
                                                        >"🗑️"</Button>
                                                    </div>
                                                </TableCell>
                                            </TableRow>
                                        }
                                    }
                                />
                            </TableBody>
                        </Table>
                        <div style="margin-top:12px;font-size:12px;color:#616161;">
                            {format!("{} tasks total · Page {} of {}",
                                page.total, page.page,
                                (page.total + page.page_size as u64 - 1) / page.page_size as u64
                            )}
                        </div>
                    }.into_any()
                }
            })}
        </div>
    }
}
