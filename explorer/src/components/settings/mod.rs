use std::iter::once;

use leptos::ev;
use leptos::html;
use leptos::html::button;
use leptos::html::dialog;
use leptos::html::div;
use leptos::html::h2;
use leptos::html::h3;
use leptos::html::input;
use leptos::html::label;
use leptos::html::span;
use leptos::prelude::*;
use leptos_scoped_style::scss_file;
use strum::IntoEnumIterator;
use zakac_i18n::SUPPORTED_LOCALES;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::compile::PLATFORM_BITS;
use crate::components::dropdown::Choice;
use crate::components::dropdown::Dropdown;
use crate::components::dropdown::dropdown;
use crate::components::icon::cross;
use crate::components::icon::gear;
use crate::components::scope::ScopedSheet;
use crate::components::scope::scoped;
use crate::font::HAS_CODE_FONT;
use crate::font::face::Face;
use crate::state::AppState;
use crate::state::lang::AUTO_LANG;
use crate::state::lang::lang_name;
use crate::state::lang::store_lang;
use crate::state::lang::stored_lang;
use crate::state::settings::FONT_SIZES;
use crate::state::settings::Theme;
use crate::state::use_state;

/// Sheet of the settings button and modal.
static SHEET: ScopedSheet =
    ScopedSheet::new("zaka-settings", scss_file!("src/components/settings/mod.sass"));

/// Element id of the modal title, which names the dialog.
const TITLE_ID: &str = "settings-modal-title";

/// The settings button and the modal it opens: the look of the page and the
/// compile options. The modal is a native dialog, so the browser centers it,
/// dims the page and keeps the focus inside; it closes on Escape, on the
/// close button and on a click on the backdrop.
///
/// # Returns
///
/// The view.
pub fn settings_menu() -> impl IntoView
{
    let state = use_state();
    let open = RwSignal::new(false);
    let root = NodeRef::<html::Div>::new();
    let trigger = button()
        .class("icon-button settings-button")
        .class(("settings-button-on", move || open.get()))
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", move || open.get().to_string())
        .attr("aria-label", t!("explorer.settings.title"))
        .title(t!("explorer.settings.title"))
        .on(ev::click, move |_| open.set(true))
        .child(gear());
    let view =
        div().class("settings").node_ref(root).child((trigger, modal(state, open)));
    scoped(&SHEET, root, view)
}

/// The dialog; it follows `open` and reports back when the browser closes it.
///
/// # Arguments
///
/// * `state` - The state.
/// * `open` - Whether the modal shows.
///
/// # Returns
///
/// The view.
fn modal(state: AppState, open: RwSignal<bool>) -> impl IntoView
{
    let host = NodeRef::<html::Dialog>::new();
    Effect::new(move |_| {
        let on = open.get();
        let Some(el) = host.get()
        else
        {
            return;
        };
        ret_if!(on == el.open());
        do_if!(on, {
            let _ = el.show_modal();
        });
        do_if!(!on, el.close());
    });
    // The card fills the dialog, so a click that lands on the dialog itself
    // is a click on the backdrop.
    let on_click = move |e: ev::MouseEvent| {
        do_if!(e.target() == e.current_target(), open.set(false))
    };
    dialog()
        .class("settings-modal")
        .node_ref(host)
        .attr("aria-labelledby", TITLE_ID)
        .on(ev::close, move |_| open.set(false))
        .on(ev::click, on_click)
        .child(card(state, open))
}

/// The card inside the dialog: a title bar, then the sections.
///
/// # Arguments
///
/// * `state` - The state.
/// * `open` - Whether the modal shows.
///
/// # Returns
///
/// The view.
fn card(state: AppState, open: RwSignal<bool>) -> impl IntoView
{
    let shut = button()
        .class("icon-button settings-close")
        .attr("aria-label", t!("explorer.settings.close_label"))
        .title(t!("explorer.settings.close"))
        .on(ev::click, move |_| open.set(false))
        .child(cross());
    let top = div().class("settings-top").child((
        h2().id(TITLE_ID).class("settings-name").child(t!("explorer.settings.title")),
        shut,
    ));
    let general = section(
        "explorer.settings.general",
        row("explorer.settings.language", lang_picker(state)),
    );
    let appearance = section(
        "explorer.settings.appearance",
        (
            row("explorer.settings.theme", theme_picker(state)),
            row("explorer.settings.ui_font", ui_font_picker(state)),
            row("explorer.settings.code_font", font_picker(state)),
            HAS_CODE_FONT
                .then(|| switch_row("explorer.settings.pda", pda_switch(state))),
            row("explorer.settings.code_size", size_picker(state)),
            switch_row("explorer.settings.italic_comments", italic_switch(state)),
        ),
    );
    let compiler = section(
        "explorer.settings.compiler",
        (
            row("explorer.settings.platform_bits", platform_picker(state)),
            switch_row("explorer.settings.fold_exprs", fold_switch(state, Fold::Exprs)),
            switch_row(
                "explorer.settings.fold_consts",
                fold_switch(state, Fold::Consts),
            ),
            switch_row(
                "explorer.settings.fold_branches",
                fold_switch(state, Fold::Branches),
            ),
        ),
    );
    let scroll = div().class("settings-scroll").child((general, appearance, compiler));
    div().class("settings-card").child((top, scroll))
}

/// A titled group of rows.
///
/// # Arguments
///
/// * `head` - Locale id of the title.
/// * `rows` - The rows.
///
/// # Returns
///
/// The view.
fn section(head: &'static str, rows: impl IntoView + 'static) -> impl IntoView
{
    div()
        .class("settings-section")
        .child((h3().class("settings-head").child(t!(head.to_string())), rows))
}

/// The text of a setting: its title and a note.
///
/// # Arguments
///
/// * `id` - Locale id of the name of the setting; the note is under
///   `<id>_note`.
///
/// # Returns
///
/// The view.
fn row_text(id: &'static str) -> impl IntoView
{
    div().class("settings-text").child((
        span().class("settings-title").child(t!(id.to_string())),
        span().class("settings-note").child(t!(format!("{id}_note"))),
    ))
}

/// One setting: its text on the left, its control on the right.
///
/// # Arguments
///
/// * `id` - Locale id of the name of the setting.
/// * `control` - The control.
///
/// # Returns
///
/// The view.
fn row(id: &'static str, control: impl IntoView + 'static) -> impl IntoView
{
    div().class("settings-row").child((row_text(id), control))
}

/// A setting with a switch; the row is a label, so a click on the text
/// flips the switch.
///
/// # Arguments
///
/// * `id` - Locale id of the name of the setting.
/// * `control` - The switch.
///
/// # Returns
///
/// The view.
fn switch_row(id: &'static str, control: impl IntoView + 'static) -> impl IntoView
{
    label().class("settings-row").child((row_text(id), control))
}

/// A dropdown of a setting.
///
/// # Arguments
///
/// * `id` - Locale id of the accessible name of the dropdown.
/// * `choices` - The choices.
/// * `value` - The value of the picked choice.
/// * `on_pick` - Called with a picked value.
///
/// # Returns
///
/// The builder, for the options that differ.
fn picker(
    id: &'static str, choices: Vec<Choice>, value: Signal<String>,
    on_pick: impl Fn(String) + Send + Sync + 'static,
) -> Dropdown
{
    Dropdown::builder()
        .label(t!(id.to_string()))
        .choices(choices)
        .value(value)
        .on_pick(Callback::new(on_pick))
        .build()
}

/// Choices for fonts, grouped as the pickers group them.
///
/// # Arguments
///
/// * `faces` - The faces, grouped in order.
///
/// # Returns
///
/// The choices.
fn face_choices(faces: impl Iterator<Item = Face>) -> Vec<Choice>
{
    let choice = |f: Face| {
        let group = f.group_label();
        Choice::builder().value(f.to_string()).label(f.label()).group(group).build()
    };
    faces.map(choice).collect()
}

/// Picker of the language of the page and the compiler messages. A pick
/// reloads the page in the new language; the URL keeps the text.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn lang_picker(state: AppState) -> impl IntoView
{
    let auto = Choice::builder()
        .value(AUTO_LANG)
        .label(t!("explorer.settings.language_auto"))
        .build();
    let named = SUPPORTED_LOCALES
        .iter()
        .map(|tag| Choice::builder().value(*tag).label(lang_name(tag)).build());
    let on_pick = move |v: String| {
        ret_if!(v == stored_lang());
        store_lang(&v);
        state.flush_link();
        let _ = window().location().reload();
    };
    let choices = once(auto).chain(named).collect();
    let current = Signal::stored(stored_lang());
    dropdown(picker("explorer.settings.language", choices, current, on_pick))
}

/// Picker of the color theme, grouped into plain and Catppuccin themes; the
/// modal focuses it when it opens.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn theme_picker(state: AppState) -> impl IntoView
{
    let choice = |t: Theme| {
        let value: &'static str = t.into();
        Choice::builder().value(value).label(t.label()).group(t.group()).build()
    };
    let current = Signal::derive(move || {
        <&'static str>::from(Theme::from_mode(&state.mode.get())).to_string()
    });
    let on_pick = move |v: String| {
        let picked = v.parse::<Theme>().unwrap_or_default();
        state.set_mode.set(picked.to_mode());
    };
    let mut spec = picker(
        "explorer.settings.theme",
        Theme::iter().map(choice).collect(),
        current,
        on_pick,
    );
    spec.autofocus = true;
    dropdown(spec)
}

/// Picker of the interface font.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn ui_font_picker(state: AppState) -> impl IntoView
{
    let current = Signal::derive(move || state.ui_font.get().to_string());
    let on_pick = move |v: String| {
        state.set_ui_font.set(v.parse().unwrap_or(Face::default_ui()));
    };
    dropdown(picker(
        "explorer.settings.ui_font",
        face_choices(Face::ui_faces()),
        current,
        on_pick,
    ))
}

/// Picker of the code font.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn font_picker(state: AppState) -> impl IntoView
{
    let current = Signal::derive(move || state.code_font.get().to_string());
    let on_pick = move |v: String| {
        state.set_code_font.set(v.parse().unwrap_or(Face::default_code()));
    };
    dropdown(picker(
        "explorer.settings.code_font",
        face_choices(Face::code_faces()),
        current,
        on_pick,
    ))
}

/// Switch that draws Burmese text in PDA10 Burmese Mono whatever the code
/// font; it is off and dimmed while PDA10 is the code font.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn pda_switch(state: AppState) -> impl IntoView
{
    let is_pda = Signal::derive(move || state.code_font.get() == Face::Pda10);
    let on = Signal::derive(move || state.pda_burmese.get() && !is_pda.get());
    switch(on, is_pda, state.set_pda_burmese)
}

/// Switch that draws comments in italics in the editor.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn italic_switch(state: AppState) -> impl IntoView
{
    switch(state.italic_comments, Signal::stored(false), state.set_italic_comments)
}

/// One of the foldings the settings switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fold
{
    /// Operations on constants.
    Exprs,
    /// Reads of constants.
    Consts,
    /// Branches on constants.
    Branches,
}

impl Fold
{
    /// The flag of this folding in a set of foldings.
    ///
    /// # Arguments
    ///
    /// * `options` - The foldings.
    ///
    /// # Returns
    ///
    /// The flag.
    fn flag(self, options: &mut FoldOptions) -> &mut bool
    {
        match self
        {
            Fold::Exprs => &mut options.exprs,
            Fold::Consts => &mut options.consts,
            Fold::Branches => &mut options.branches,
        }
    }
}

/// Switch of one folding of the IR.
///
/// # Arguments
///
/// * `state` - The state.
/// * `fold` - The folding.
///
/// # Returns
///
/// The view.
fn fold_switch(state: AppState, fold: Fold) -> impl IntoView
{
    let on = Signal::derive(move || *fold.flag(&mut state.options.get().fold));
    let set = SignalSetter::map(move |b| {
        state.options.update(|o| *fold.flag(&mut o.fold) = b)
    });
    switch(on, Signal::stored(false), set)
}

/// A switch: a checkbox drawn as a track with a knob.
///
/// # Arguments
///
/// * `on` - Whether it is on.
/// * `off_limits` - Whether it is dimmed and does not flip.
/// * `set` - Called with the new state when it flips.
///
/// # Returns
///
/// The view.
fn switch(
    on: Signal<bool>, off_limits: Signal<bool>, set: impl Into<SignalSetter<bool>>,
) -> impl IntoView
{
    let set: SignalSetter<bool> = set.into();
    let check = input()
        .r#type("checkbox")
        .prop("checked", on)
        .prop("disabled", off_limits)
        .on(ev::change, move |ev| set.set(event_target_checked(&ev)));
    span().class("settings-switch").child((check, span().class("switch-track")))
}

/// Picker of the code font size.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn size_picker(state: AppState) -> impl IntoView
{
    let choice = |n: &u32| {
        let label = t!("explorer.settings.code_size_choice", n = n);
        Choice::builder().value(n.to_string()).label(label).build()
    };
    let current = Signal::derive(move || state.font_size.get().to_string());
    let on_pick =
        move |v: String| state.set_font_size.set(v.parse().unwrap_or_default());
    dropdown(picker(
        "explorer.settings.code_size",
        FONT_SIZES.iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of the bit count of the platform-sized integer kinds.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn platform_picker(state: AppState) -> impl IntoView
{
    let choice = |b: &u16| {
        let label = t!("explorer.settings.platform_bits_choice", n = b);
        Choice::builder().value(b.to_string()).label(label).build()
    };
    let current = Signal::derive(move || state.options.get().platform_bits.to_string());
    let on_pick = move |v: String| {
        let bits = v.parse().unwrap_or(64);
        state.options.update(|o| o.platform_bits = bits);
    };
    dropdown(picker(
        "explorer.settings.platform_bits",
        PLATFORM_BITS.iter().map(choice).collect(),
        current,
        on_pick,
    ))
}
