#![recursion_limit = "1024"]

mod app;
mod components;
mod context;
mod model;
mod router;
mod routes;
mod services;
mod utils;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Stable re-exports for `tests/web.rs` (integration tests link without `cfg(test)`).
pub mod browser_test_api {
    pub use crate::context::ThemePreference;
    pub use crate::model::{
        ContactSubmission, ContentCollectionSummary, ContentListResponse, Login, MediaFeedItem,
        MediaFeedResponse, ServerResponse, User,
    };
    pub use crate::routes::AppRoutes;
    pub use crate::services::api_errors;
    pub use crate::utils::has_explicit_offset;
}

#[wasm_bindgen(start)]
pub fn run_app() {
    utils::set_panic_hook();
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    yew::Renderer::<app::App>::new().render();
}
