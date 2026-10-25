use leptos::prelude::window;
use wasm_bindgen::JsValue;

/// Read the URL hash.
///
/// # Returns
///
/// The hash without the leading `#`, or `None` when there is none.
pub fn read_hash() -> Option<String>
{
    let hash = window().location().hash().ok()?;
    let hash = hash.trim_start_matches('#');
    (!hash.is_empty()).then(|| hash.to_string())
}

/// Replace the URL hash without reloading the page or adding a back step.
///
/// # Arguments
///
/// * `hash` - The hash without the leading `#`.
pub fn write_hash(hash: &str)
{
    let Ok(steps) = window().history()
    else
    {
        return;
    };
    let _ = steps.replace_state_with_url(&JsValue::NULL, "", Some(&format!("#{hash}")));
}
