// ─────────────────────────────────────────────────────────────
// crates/frontend/src/components/nav.rs
// ─────────────────────────────────────────────────────────────
use leptos::prelude::*;
use thaw::Icon;
use icondata as i;

#[component]
pub fn SideNav() -> impl IntoView {
    view! {
        <nav class="side-nav" aria-label="Main navigation">
            <NavBtn icon=i::LuCheckSquare label="Tasks" href="/tasks" active=true />
            <NavBtn icon=i::LuUsers       label="Contacts" href="/contacts" active=false />
            <NavBtn icon=i::LuBarChart2   label="Reports" href="/reports" active=false />
            <div style="flex:1" />
            <NavBtn icon=i::LuSettings    label="Settings" href="/settings" active=false />
        </nav>
    }
}

#[component]
fn NavBtn(
    icon:   icondata::Icon,
    label:  &'static str,
    href:   &'static str,
    active: bool,
) -> impl IntoView {
    view! {
        <a
            href=href
            class=move || if active { "nav-icon-btn active" } else { "nav-icon-btn" }
            title=label
            aria-label=label
        >
            <Icon icon width="20px" height="20px" />
        </a>
    }
}
