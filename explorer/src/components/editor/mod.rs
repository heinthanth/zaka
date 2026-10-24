use leptos::html;
use leptos::html::div;
use leptos::html::p;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::GlobalStyles;
use leptos_scoped_style::component::GlobalStylesProps;
use leptos_scoped_style::scss_file;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::components::scope::SLOT_MARK;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::font::load_code_font;
use crate::font::load_face;
use crate::monaco::Editor;
use crate::state::AppState;
use crate::state::settings::Theme;
use crate::state::use_state;

/// Sheet of the editor pane.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-editor", scss_file!("src/components/editor/mod.sass"));

/// Token colors on Monaco's decoration spans; global, since Monaco owns those
/// nodes.
static MONACO: ComponentStyle = ComponentStyle::global_css(
    "zaka-monaco",
    scss_file!("src/components/editor/monaco.sass"),
);

/// The source editor; Monaco fills its host once it has loaded. The host is a
/// slot, so the scope tags it but leaves Monaco's nodes alone.
///
/// # Returns
///
/// The view.
pub fn editor_pane() -> impl IntoView
{
    let state = use_state();
    let host = NodeRef::<html::Div>::new();
    Effect::new(move |_| {
        let Some(el) = host.get()
        else
        {
            return;
        };
        ret_if!(state.editor.with_value(Option::is_some));
        spawn_local(mount(state, el.unchecked_into()));
    });
    let wait = move || {
        let pending = !state.mounted.get();
        pending.then(|| p().class("editor-wait").child(t!("explorer.editor.loading")))
    };
    let root = NodeRef::<html::Div>::new();
    let monaco_host = div().class("editor-host").attr(SLOT_MARK, "").node_ref(host);
    let view = div().class("editor-pane").node_ref(root).child((monaco_host, wait));
    let colors = GlobalStyles(GlobalStylesProps::builder().style(&MONACO).build());
    (colors, scoped(&SHEET, root, view))
}

/// Create the editor and connect it to the state.
///
/// # Arguments
///
/// * `state` - The state.
/// * `host` - The element the editor fills.
async fn mount(state: AppState, host: HtmlElement)
{
    let text = state.source.get_untracked();
    let dark = Theme::is_dark(&state.resolved.get_untracked());
    let (stack, size) = state.font_untracked();
    // Monaco measures the font when it is created, so the fonts go first.
    do_if!(let Err(err) = load_code_font().await, web_sys::console::error_1(&err));
    load_face(state.code_font.get_untracked(), size).await;
    let Some(editor) = Editor::create(&host, &text, dark, (&stack, size)).await
    else
    {
        return;
    };
    editor.on_edit(move |text| state.source.set(text));
    editor.on_caret(move |off16| state.caret_token.set(state.token_at(off16)));
    editor.on_point(move |off16| {
        state.mouse_token.set(off16.and_then(|off16| state.token_at(off16)));
    });
    state.editor.set_value(Some(editor));
    state.mounted.set(true);
}
