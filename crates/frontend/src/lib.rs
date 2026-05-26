// crates/frontend/src/lib.rs
//
// WASM entry point. Initialises logging and mounts the Leptos app.

use leptos::prelude::*;
use wasm_bindgen::prelude::*;

mod app;
mod api;
mod components;
mod pages;

pub use app::App;

#[wasm_bindgen(start)]
pub fn main() {
    // Panic messages appear in the browser console
    console_error_panic_hook::set_once();

    // Route log::* macros to console.log / console.error
    console_log::init_with_level(log::Level::Debug)
        .expect("Failed to init console logger");

    mount_to_body(App);
}
