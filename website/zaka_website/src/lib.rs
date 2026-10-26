// The views of a whole section nest deep in the compiler.
#![recursion_limit = "256"]

pub mod app;
pub mod assets;
pub mod components;
pub mod manual;
#[cfg(feature = "client")]
mod motion;

zakac_i18n::init!();

/// Env var with the address the site is served from, such as
/// `https://zaka-lang.org`; links that must stand alone, as the cover card a
/// chat app or a feed fetches, start with it. A build for the web must set it.
pub const ORIGIN_VAR: &str = "ZAKA_SITE_ORIGIN";

/// Start the islands in the browser, then the motion code.
#[cfg(feature = "client")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate()
{
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
    motion::start();
}
