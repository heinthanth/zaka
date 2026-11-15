// The views of a whole section nest deep in the compiler.
#![recursion_limit = "256"]

pub mod app;
pub mod assets;
pub mod book;
pub mod components;
#[cfg(feature = "client")]
mod motion;

zakac_i18n::init!();

/// Env var with the address the site is served from, such as
/// `https://zaka-lang.org`; links that must stand alone, as the cover card a
/// chat app or a feed fetches, start with it. A build for the web must set it.
pub const ORIGIN_VAR: &str = "ZAKA_SITE_ORIGIN";

/// The address in [`ORIGIN_VAR`], with no trailing slash; a blank value counts
/// as unset, as CI passes one when the variable is missing.
///
/// # Returns
///
/// The address, or `None` when the variable is unset or blank.
pub fn set_origin() -> Option<String>
{
    let origin = std::env::var(ORIGIN_VAR).ok()?;
    let origin = origin.trim().trim_end_matches('/');
    (!origin.is_empty()).then(|| origin.to_string())
}

/// Start the islands in the browser, then the motion code.
#[cfg(feature = "client")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate()
{
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
    motion::start();
}
