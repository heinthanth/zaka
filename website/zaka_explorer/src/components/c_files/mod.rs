use std::borrow::Cow;

use leptos::ev;
use leptos::html;
use leptos::html::aside;
use leptos::html::button;
use leptos::html::div;
use leptos::html::h3;
use leptos::html::span;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_scoped_style::ComponentStyle;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use zaka_web_ui_common::icon::chevron;
use zaka_web_ui_common::sheet;
use zaka_web_ui_common::theme::Theme;
use zakac_codegen_target_c::driver::RUNTIME;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::compile::Compiled;
use crate::components::check_view::skipped;
use crate::monaco::Viewer;
use crate::state::AppState;
use crate::state::use_state;

// The sheet, its classes renamed after a hash: `STYLE_SHEET` and `ClassName`.
turf::style_sheet!("src/components/c_files/mod.sass");

/// Sheet of the C viewer and the list of the C files.
static SHEET: ComponentStyle = ComponentStyle::global_css("zaka-c-files", STYLE_SHEET);

/// The viewer slot of a pane: blank until Monaco has loaded.
type Slot = StoredValue<Option<Viewer>, LocalStorage>;

/// The C file the list marks open, in Monaco with no edits, under a head with
/// its name; or a note when the C target wrote no file.
///
/// # Returns
///
/// The view.
pub fn c_viewer() -> impl IntoView
{
    let state = use_state();
    let note = Memo::new(move |_| state.compiled.with(c_note));
    let content = move || match note.get()
    {
        None => viewer_pane(state).into_any(),
        Some((title, text)) => div()
            .class(ClassName::C_VIEWER_SKIPPED)
            .child(skipped(title, text))
            .into_any(),
    };
    let view = div().class(ClassName::C_VIEWER).child(content);
    (sheet(&SHEET), view)
}

/// The note in place of the viewer, when the C target wrote no file.
///
/// # Arguments
///
/// * `c` - The compile run.
///
/// # Returns
///
/// The reason and the fix, or `None` when there are files to show.
fn c_note(c: &Compiled) -> Option<(Cow<'static, str>, Cow<'static, str>)>
{
    let skipped_note =
        (t!("explorer.codegen.skipped_title"), t!("explorer.codegen.skipped_note"));
    ret_if!(!c.emitted, Some(skipped_note));
    let missing = c.c_missing.clone().map(Cow::Owned);
    ret_if!(let Some(text) = missing, Some((t!("explorer.c_files.missing_title"), text)));
    let kinds = (t!("explorer.c_files.kinds_title"), t!("explorer.c_files.kinds_note"));
    tern!(c.c_files.is_empty(), Some(kinds), None)
}

/// The head with the name of the open file, over the viewer.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn viewer_pane(state: AppState) -> impl IntoView
{
    let host = NodeRef::<html::Div>::new();
    let slot: Slot = StoredValue::new_local(None);
    let text = Memo::new(move |_| file_text(state, &open_name(state)));
    Effect::new(move |_| {
        let Some(el) = host.get()
        else
        {
            return;
        };
        ret_if!(slot.with_value(Option::is_some));
        spawn_local(mount(state, slot, el.unchecked_into(), text));
    });
    Effect::new(move |_| {
        let shown = text.get();
        slot.with_value(|v| do_if!(let Some(v) = v, v.set_text(&shown)));
    });
    Effect::new(move |_| {
        let (pair, size) = (state.look.code_pair(), state.look.font_size.get());
        let stack = pair.code_stack();
        slot.with_value(|v| do_if!(let Some(v) = v, v.set_font(&stack, size)));
    });
    on_cleanup(move || {
        slot.try_with_value(|v| do_if!(let Some(v) = v, v.dispose()));
    });
    let head = div().class(ClassName::C_VIEWER_HEAD).child(move || open_name(state));
    let monaco_host = div().class(ClassName::C_VIEWER_HOST).node_ref(host);
    div().class(ClassName::C_VIEWER_PANE).child((head, monaco_host))
}

/// Create the viewer once Monaco has loaded, with the text of the open
/// file. The pane can leave the page while Monaco loads; the viewer is then
/// freed at once.
///
/// # Arguments
///
/// * `state` - The state.
/// * `slot` - Where the viewer goes.
/// * `host` - The element the viewer fills.
/// * `text` - The text of the open file.
async fn mount(state: AppState, slot: Slot, host: HtmlElement, text: Memo<String>)
{
    let dark = Theme::is_dark(&state.look.resolved.get_untracked());
    let (stack, size) = state.font_untracked();
    let first = text.try_get_untracked().unwrap_or_default();
    let Some(made) = Viewer::create(&host, &first, dark, (&stack, size)).await
    else
    {
        return;
    };
    made.set_text(&text.try_get_untracked().unwrap_or_default());
    let kept = slot.try_update_value(|v| *v = Some(made.clone()));
    do_if!(kept.is_none(), made.dispose());
}

/// The list of the C files, as the file tree of an editor: the files the
/// program makes, then the runtime files. A click opens a file in the viewer.
///
/// # Returns
///
/// The view.
pub fn c_file_list() -> impl IntoView
{
    let state = use_state();
    let names = Memo::new(move |_| {
        state.compiled.with(|c| c.c_files.iter().map(|f| f.name.clone()).collect())
    });
    let made = move || group(state, t!("explorer.c_files.program"), names.get());
    let runtime = RUNTIME.iter().map(|(name, _)| name.to_string()).collect();
    let view = aside()
        .class(ClassName::C_FILE_LIST)
        .attr("aria-label", t!("explorer.c_files.label"))
        .child((
            h3().class(ClassName::C_FILE_LIST_TITLE)
                .child(t!("explorer.c_files.title")),
            made,
            group(state, t!("explorer.c_files.runtime"), runtime),
        ));
    (sheet(&SHEET), view)
}

/// A folder of the list: a head that opens and shuts it, then its files.
///
/// # Arguments
///
/// * `state` - The state.
/// * `title` - The name of the folder.
/// * `names` - The names of its files.
///
/// # Returns
///
/// The view.
fn group(state: AppState, title: Cow<'static, str>, names: Vec<String>)
-> impl IntoView
{
    let open = RwSignal::new(true);
    let head = button()
        .class(ClassName::C_GROUP_HEAD)
        .r#type("button")
        .attr("aria-expanded", move || open.get().to_string())
        .on(ev::click, move |_| open.update(|o| *o = !*o))
        .child((chevron(), span().child(title)));
    let files = names.into_iter().map(|n| file_button(state, n)).collect_view();
    let inside = div().class(ClassName::C_GROUP_FILES).child(files);
    div()
        .class(ClassName::C_GROUP)
        .class((ClassName::C_GROUP_SHUT, move || !open.get()))
        .child((head, inside))
}

/// One file of the list, marked while the viewer shows it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `name` - The file name.
///
/// # Returns
///
/// The view.
fn file_button(state: AppState, name: String) -> impl IntoView
{
    let header = name.ends_with(".h");
    let mark = span()
        .class(ClassName::C_FILE_MARK)
        .class((ClassName::C_FILE_MARK_H, header))
        .attr("aria-hidden", "true")
        .child(tern!(header, "h", "c"));
    let mine = name.clone();
    let is_open = move || open_name(state) == mine;
    let picked = name.clone();
    button()
        .class(ClassName::C_FILE)
        .r#type("button")
        .class((ClassName::C_FILE_OPEN, is_open.clone()))
        .attr("aria-current", move || is_open().then_some("true"))
        .on(ev::click, move |_| state.c_file.set(picked.clone()))
        .child((mark, span().child(name)))
}

/// The name of the file the viewer shows: the chosen one while the list holds
/// it, else the first file the program makes.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The name, blank when the C target wrote no file.
fn open_name(state: AppState) -> String
{
    let chosen = state.c_file.get();
    let runtime = RUNTIME.iter().any(|(name, _)| *name == chosen);
    state.compiled.with(|c| {
        let made = c.c_files.iter().any(|f| f.name == chosen);
        let first = c.c_files.first().map(|f| f.name.clone()).unwrap_or_default();
        tern!(made || runtime, chosen, first)
    })
}

/// The text of a C file: one the program makes, or a runtime file.
///
/// # Arguments
///
/// * `state` - The state.
/// * `name` - The file name.
///
/// # Returns
///
/// The text, blank for a name the list does not hold.
fn file_text(state: AppState, name: &str) -> String
{
    let made = state
        .compiled
        .with(|c| c.c_files.iter().find(|f| f.name == name).map(|f| f.text.clone()));
    let runtime =
        || RUNTIME.iter().find(|(n, _)| *n == name).map(|(_, t)| t.to_string());
    made.or_else(runtime).unwrap_or_default()
}
