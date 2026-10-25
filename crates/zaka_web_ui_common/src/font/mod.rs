use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use js_sys::Promise;
use leptos::prelude::document;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::FontFace;
use web_sys::FontFaceDescriptors;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

pub mod face;
pub mod mask;

use crate::font::face::Face;
use crate::font::face::Source;
use crate::font::mask::SEED;
use crate::font::mask::xor_mask;

/// Folder of the web fonts when the page sets none: `fonts/` beside the page,
/// where the explorer build copies them.
const DEFAULT_ROOT: &str = "fonts/";

/// Whether the build holds the code font; `build.rs` leaves it out when no
/// secret to open it is set.
pub const HAS_CODE_FONT: bool = cfg!(pda10);

/// Font name of the code font.
pub const CODE_FONT: &str = "PDA10 Burmese Mono";

/// Font name of the same font limited to the Burmese range; first in a stack,
/// it draws Burmese text and leaves the rest to the chosen font. It needs a
/// name of its own: under [`CODE_FONT`] the whole face would win.
pub const BURMESE_FONT: &str = "PDA10 Burmese Mono (Burmese)";

/// The Burmese range: the Myanmar block, the joiners and the dotted circle.
pub const BURMESE_RANGE: &str = "U+1000-109F, U+200C-200D, U+25CC";

/// Text that pulls in both the Burmese and the Latin subset of a web font.
const PROBE: &str = "\u{1000}A";

/// Folder of the web fonts, as the page sets it.
static ROOT: OnceLock<&'static str> = OnceLock::new();

thread_local! {
    /// The load of each sheet or font file the page has asked for, kept
    /// under its URL; a second request waits on the first.
    static STARTED: RefCell<HashMap<String, Promise>> = RefCell::new(HashMap::new());

    /// The file of each [`Source::Shipped`] face, as the app names it.
    static SHIPPED: RefCell<HashMap<Face, &'static str>> = RefCell::new(HashMap::new());
}

/// The code font as `build.rs` masked it; blank without [`HAS_CODE_FONT`].
pub static SCRAMBLED: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/code_font.bin"));

/// The code font file, unmasked.
///
/// # Returns
///
/// The TrueType file.
pub fn font_raw() -> Vec<u8>
{
    let mut raw = SCRAMBLED.to_vec();
    xor_mask(&mut raw, SEED);
    raw
}

/// Register the code font with the page straight from the wasm, twice: whole
/// under [`CODE_FONT`], and limited to [`BURMESE_RANGE`] under
/// [`BURMESE_FONT`]. No URL or request is involved, and the faces cannot be
/// read back.
///
/// # Returns
///
/// `Ok` once both faces are loaded and added, at once when the build holds no
/// code font, or the error of the browser.
pub async fn load_code_font() -> Result<(), JsValue>
{
    ret_if!(!HAS_CODE_FONT, Ok(()));
    let raw = font_raw();
    add_face(FontFace::new_with_u8_array(CODE_FONT, &raw)?).await?;
    let burmese = FontFaceDescriptors::new();
    burmese.set_unicode_range(BURMESE_RANGE);
    add_face(FontFace::new_with_u8_array_and_descriptors(BURMESE_FONT, &raw, &burmese)?)
        .await
}

/// Load a face and add it to the page.
///
/// # Arguments
///
/// * `face` - The face.
///
/// # Returns
///
/// `Ok` once it is added, or the error of the browser.
async fn add_face(face: FontFace) -> Result<(), JsValue>
{
    JsFuture::from(face.load()?).await?;
    document().fonts().add(&face)
}

/// Set the folder the web fonts load from, once, before the first load.
///
/// # Arguments
///
/// * `root` - The folder URL, with a trailing slash.
pub fn set_font_root(root: &'static str) { _ = ROOT.set(root) }

/// Name the file of a face the app ships under a hashed name, before the
/// first load of the face.
///
/// # Arguments
///
/// * `face` - A face with a [`Source::Shipped`] source.
/// * `url` - The address of the file.
pub fn set_face_file(face: Face, url: &'static str)
{
    SHIPPED.with(|map| map.borrow_mut().insert(face, url));
}

/// The folder the web fonts load from.
///
/// # Returns
///
/// The folder URL, with a trailing slash.
pub fn font_root() -> &'static str { ROOT.get().copied().unwrap_or(DEFAULT_ROOT) }

/// Wait until a font family is loaded at a size, so that a measure of the
/// text sees the real glyphs. A failure is logged and the fallbacks take
/// over.
///
/// # Arguments
///
/// * `family` - The CSS font family.
/// * `size` - The size in px.
pub async fn ready_font(family: &str, size: u32)
{
    let query = format!("{size}px \"{family}\"");
    let wait = document().fonts().load_with_text(&query, PROBE);
    do_if!(let Err(err) = JsFuture::from(wait).await, web_sys::console::error_1(&err));
}

/// Prepare a face to draw: add its sheets or files to the page the first
/// time it is picked, then wait until it is loaded at a size. Failures are
/// logged and the fallbacks take over.
///
/// # Arguments
///
/// * `face` - The face.
/// * `size` - The size in px.
pub async fn load_face(face: Face, size: u32)
{
    let root = font_root();
    let loads = match face.source()
    {
        Source::Native => return,
        Source::Embedded => Vec::new(),
        Source::Sheets { weights, .. } =>
        {
            let href =
                |w: &u16| format!("{root}{}", face.sheet_path(*w).unwrap_or_default());
            weights.iter().map(|w| once(&href(w), add_sheet)).collect()
        }
        Source::Shipped =>
        {
            let file = SHIPPED.with(|map| map.borrow().get(&face).copied());
            let add = |url: &str| once(url, |u| add_file(face.family(), 400, u));
            file.map(add).into_iter().collect()
        }
        Source::Files(files) =>
        {
            let add = |(w, path): &(u16, &str)| {
                once(&format!("{root}{path}"), |u| add_file(face.family(), *w, u))
            };
            files.iter().map(add).collect()
        }
    };
    do_for!(load in loads, log_failure(wait(load).await));
    ready_font(face.family(), size).await;
}

/// Start a load once per URL.
///
/// # Arguments
///
/// * `url` - The URL of the sheet or file.
/// * `start` - Start the load; it runs the first time alone.
///
/// # Returns
///
/// The load, or the error of the browser.
fn once(
    url: &str, start: impl FnOnce(&str) -> Result<Promise, JsValue>,
) -> Result<Promise, JsValue>
{
    let known = STARTED.with(|map| map.borrow().get(url).cloned());
    ret_if!(let Some(load) = known, Ok(load));
    let load = start(url)?;
    STARTED.with(|map| map.borrow_mut().insert(url.to_string(), load.clone()));
    Ok(load)
}

/// Add a stylesheet to the page.
///
/// # Arguments
///
/// * `href` - Its URL.
///
/// # Returns
///
/// A promise that settles once the sheet is parsed, or the error of the
/// browser.
fn add_sheet(href: &str) -> Result<Promise, JsValue>
{
    let link = document().create_element("link")?;
    link.set_attribute("rel", "stylesheet")?;
    link.set_attribute("href", href)?;
    let loaded = Promise::new(&mut |done, fail| {
        let _ = link.add_event_listener_with_callback("load", &done);
        let _ = link.add_event_listener_with_callback("error", &fail);
    });
    let head =
        document().head().ok_or_else(|| JsValue::from_str("the page has no head"))?;
    head.append_child(&link)?;
    Ok(loaded)
}

/// Add a font file to the page as one weight of a family.
///
/// # Arguments
///
/// * `family` - The CSS font family.
/// * `weight` - The weight of the file.
/// * `url` - Its URL.
///
/// # Returns
///
/// The load of the file, or the error of the browser.
fn add_file(family: &str, weight: u16, url: &str) -> Result<Promise, JsValue>
{
    let descriptors = FontFaceDescriptors::new();
    descriptors.set_weight(&weight.to_string());
    let font = FontFace::new_with_str_and_descriptors(
        family,
        &format!("url({url})"),
        &descriptors,
    )?;
    document().fonts().add(&font)?;
    font.load()
}

/// Wait for a load.
///
/// # Arguments
///
/// * `load` - The load, or the error that kept it from starting.
///
/// # Returns
///
/// `Ok` once it settles well, or the error.
async fn wait(load: Result<Promise, JsValue>) -> Result<(), JsValue>
{
    JsFuture::from(load?).await.map(|_| ())
}

/// Log a failed load to the console.
///
/// # Arguments
///
/// * `result` - The result of the load.
fn log_failure(result: Result<(), JsValue>)
{
    do_if!(let Err(err) = result, web_sys::console::error_1(&err));
}
