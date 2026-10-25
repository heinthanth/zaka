//! The Zaka website: pages rendered to HTML when the site is built, with
//! islands of Rust compiled to wasm for the parts a visitor works with.

// The views of a whole section nest deep in the compiler.
#![recursion_limit = "256"]

pub mod app;
pub mod assets;
pub mod components;
#[cfg(feature = "client")]
mod motion;

zakac_i18n::init!();

/// Folder of the web fonts: the explorer build ships them, under the same
/// origin.
pub const FONT_ROOT: &str = "/explorer/fonts/";

/// Start the islands in the browser, then the motion code.
#[cfg(feature = "client")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate()
{
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
    motion::start();
}
