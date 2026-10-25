use leptos::html;
use leptos::html::div;
use leptos::html::p;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_scoped_style::ComponentStyle;
use leptos_scoped_style::scss_file;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use zaka_web_ui_common::font::load_pair;
use zaka_web_ui_common::sheet;
use zaka_web_ui_common::theme::Theme;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::font::load_code_font;
use crate::monaco::Editor;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/editor/mod.sass");

/// Sheet of the editor pane.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-editor", STYLE_SHEET);

/// Token colors on Monaco's decoration spans; global, since Monaco owns those
/// nodes.
static MONACO: ComponentStyle = ComponentStyle::global_css(
    "zaka-monaco",
    scss_file!("src/components/editor/monaco.sass"),
);

/// The source editor; Monaco fills its host once it has loaded.
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
        pending.then(|| {
            p().class(ClassName::EDITOR_WAIT).child(t!("explorer.editor.loading"))
        })
    };
    let monaco_host = div().class(ClassName::EDITOR_HOST).node_ref(host);
    let view = div().class(ClassName::EDITOR_PANE).child((monaco_host, wait));
    (sheet(&MONACO), sheet(&SHEET), view)
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
    let dark = Theme::is_dark(&state.look.resolved.get_untracked());
    let (stack, size) = state.font_untracked();
    // Monaco measures the font when it is created, so the fonts go first.
    do_if!(let Err(err) = load_code_font().await, web_sys::console::error_1(&err));
    load_pair(untrack(|| state.look.code_pair()), size).await;
    let Some(editor) = Editor::create(&host, &text, dark, (&stack, size)).await
    else
    {
        return;
    };
    editor.on_edit(move |text| {
        state.source.set(text);
        state.read_break_lines();
    });
    editor.on_gutter_click(move |line| state.toggle_line(line));
    editor.on_caret(move |off16| state.caret_token.set(state.token_at(off16)));
    editor.on_point(move |off16| {
        state.mouse_token.set(off16.and_then(|off16| state.token_at(off16)));
    });
    state.editor.set_value(Some(editor));
    state.mounted.set(true);
}
