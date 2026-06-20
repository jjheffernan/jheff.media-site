#![recursion_limit = "1024"]

mod app;
mod components;
mod context;
mod model;
mod router;
mod routes;
mod utils;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn run_app() {
    utils::set_panic_hook();
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    yew::Renderer::<app::App>::new().render();
}
