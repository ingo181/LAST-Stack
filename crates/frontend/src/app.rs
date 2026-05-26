// crates/frontend/src/app.rs
//
// Root component — Thaw ConfigProvider + Leptos Router.

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use thaw::{ConfigProvider, Theme};

use crate::components::nav::SideNav;
use crate::pages::{TaskListPage, NotFoundPage};

#[component]
pub fn App() -> impl IntoView {
    // Thaw light theme — matches Fluent Blue Light from DevExtreme inspiration
    let theme = RwSignal::new(Theme::light());

    view! {
        <ConfigProvider theme>
            <Router>
                <div class="app-shell">
                    <SideNav />
                    <main class="app-content">
                        <Routes fallback=|| view! { <NotFoundPage /> }>
                            <Route path=path!("/")        view=TaskListPage />
                            <Route path=path!("/tasks")   view=TaskListPage />
                        </Routes>
                    </main>
                </div>
            </Router>
        </ConfigProvider>
    }
}

