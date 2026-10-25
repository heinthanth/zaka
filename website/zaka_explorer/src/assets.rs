use leptos::prelude::document;
use wasm_bindgen::JsValue;
use zakac_rust_sugars::do_if;

use crate::font::face::Face;
use crate::font::set_face_file;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// The icons of the tab for each OS color scheme; `index.html` sets the
/// default, which Trunk hashes, so the first paint has one.
const ICONS: [(&str, &str); 2] = [
    (LOGO_LIGHT, "(prefers-color-scheme: light)"),
    (LOGO_DARK, "(prefers-color-scheme: dark)"),
];

/// Hand the hashed files to the page before it mounts: the tab icons, and the
/// Sauce Code Pro and Myanmar Sabae files to the font loader.
pub fn ship()
{
    set_face_file(Face::SauceHtmm, SAUCE_FONT);
    set_face_file(Face::Sabae, SABAE_FONT);
    ICONS.iter().for_each(|&(href, media)| {
        let added = add_icon(href, media);
        do_if!(let Err(err) = added, web_sys::console::error_1(&err));
    });
}

/// Add one tab icon to the head of the page.
///
/// # Arguments
///
/// * `href` - The address of the SVG.
/// * `media` - The media rule it is for.
///
/// # Returns
///
/// `Ok` once added, or the error of the browser.
fn add_icon(href: &str, media: &str) -> Result<(), JsValue>
{
    let link = document().create_element("link")?;
    link.set_attribute("rel", "icon")?;
    link.set_attribute("type", "image/svg+xml")?;
    link.set_attribute("href", href)?;
    link.set_attribute("media", media)?;
    let head =
        document().head().ok_or_else(|| JsValue::from_str("the page has no head"))?;
    head.append_child(&link).map(|_| ())
}
