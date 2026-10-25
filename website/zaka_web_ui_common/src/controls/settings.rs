use leptos::ev;
use leptos::html::button;
use leptos::html::div;
use leptos::html::h3;
use leptos::html::input;
use leptos::html::label;
use leptos::html::span;
use leptos::prelude::*;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;

use crate::controls::ClassName;
use crate::controls::dropdown::Choice;
use crate::controls::dropdown::Dropdown;
use crate::controls::dropdown::dropdown;
use crate::controls::modal::Modal;
use crate::controls::modal::modal;
use crate::controls::pickers::theme_picker;
use crate::font::face::Face;
use crate::font::face::Fallback;
use crate::icon::gear;
use crate::look::FONT_SIZES;
use crate::look::Look;

/// Element id of the modal title, which names the dialog.
const TITLE_ID: &str = "settings-modal-title";

/// The settings button and the modal it opens.
///
/// # Arguments
///
/// * `content` - The sections of the modal.
///
/// # Returns
///
/// The view.
pub fn settings_menu(content: impl IntoView + 'static) -> impl IntoView
{
    let open = RwSignal::new(false);
    let title = t!("web_ui.settings.title").to_string();
    let trigger = button()
        .class(format!("{} {}", ClassName::ICON_BUTTON, ClassName::SETTINGS_BUTTON))
        .class((ClassName::SETTINGS_BUTTON_ON, move || open.get()))
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", move || open.get().to_string())
        .attr("aria-label", title.clone())
        .title(title.clone())
        .on(ev::click, move |_| open.set(true))
        .child(gear());
    let spec = Modal::builder()
        .open(open)
        .class(ClassName::SETTINGS_MODAL)
        .title_id(TITLE_ID)
        .title(title)
        .close_label(t!("web_ui.settings.close_label"))
        .build();
    div().child((trigger, modal(spec, content)))
}

/// The appearance section: theme, the Burmese and fallback fonts of
/// interface and code text, code size and italic comments.
///
/// # Arguments
///
/// * `look` - The look.
///
/// # Returns
///
/// The view.
pub fn appearance(look: Look) -> impl IntoView
{
    section(
        t!("web_ui.settings.appearance"),
        (
            row(
                t!("web_ui.settings.theme"),
                t!("web_ui.settings.theme_note"),
                theme_picker(look.theme(), true),
            ),
            row(
                t!("web_ui.settings.ui_burmese"),
                t!("web_ui.settings.ui_burmese_note"),
                burmese_picker(
                    t!("web_ui.settings.ui_burmese"),
                    look.ui_burmese,
                    look.set_ui_burmese,
                ),
            ),
            row(
                t!("web_ui.settings.ui_fallback"),
                t!("web_ui.settings.ui_fallback_note"),
                fallback_picker(
                    t!("web_ui.settings.ui_fallback"),
                    Face::ui_faces(),
                    look.ui_fallback,
                    look.set_ui_fallback,
                ),
            ),
            row(
                t!("web_ui.settings.code_burmese"),
                t!("web_ui.settings.code_burmese_note"),
                burmese_picker(
                    t!("web_ui.settings.code_burmese"),
                    look.code_burmese,
                    look.set_code_burmese,
                ),
            ),
            row(
                t!("web_ui.settings.code_fallback"),
                t!("web_ui.settings.code_fallback_note"),
                fallback_picker(
                    t!("web_ui.settings.code_fallback"),
                    Face::code_faces(),
                    look.code_fallback,
                    look.set_code_fallback,
                ),
            ),
            row(
                t!("web_ui.settings.code_size"),
                t!("web_ui.settings.code_size_note"),
                size_picker(look),
            ),
            switch_row(
                t!("web_ui.settings.italic_comments"),
                t!("web_ui.settings.italic_comments_note"),
                italic_switch(look),
            ),
        ),
    )
}

/// A titled group of rows.
///
/// # Arguments
///
/// * `head` - The title.
/// * `rows` - The rows.
///
/// # Returns
///
/// The view.
pub fn section(head: impl Into<String>, rows: impl IntoView + 'static)
-> impl IntoView
{
    div().child((h3().class(ClassName::SETTINGS_HEAD).child(head.into()), rows))
}

/// The text of a setting: its title and a note.
///
/// # Arguments
///
/// * `title` - The name of the setting.
/// * `note` - The note under it.
///
/// # Returns
///
/// The view.
fn row_text(title: String, note: String) -> impl IntoView
{
    div().class(ClassName::SETTINGS_TEXT).child((
        span().class(ClassName::SETTINGS_TITLE).child(title),
        span().class(ClassName::SETTINGS_NOTE).child(note),
    ))
}

/// One setting: its text on the left, its control on the right.
///
/// # Arguments
///
/// * `title` - The name of the setting.
/// * `note` - The note under it.
/// * `control` - The control.
///
/// # Returns
///
/// The view.
pub fn row(
    title: impl Into<String>, note: impl Into<String>, control: impl IntoView + 'static,
) -> impl IntoView
{
    div()
        .class(ClassName::SETTINGS_ROW)
        .child((row_text(title.into(), note.into()), control))
}

/// A setting with a switch; the row is a label, so a click on the text
/// flips the switch.
///
/// # Arguments
///
/// * `title` - The name of the setting.
/// * `note` - The note under it.
/// * `control` - The switch.
///
/// # Returns
///
/// The view.
pub fn switch_row(
    title: impl Into<String>, note: impl Into<String>, control: impl IntoView + 'static,
) -> impl IntoView
{
    label()
        .class(ClassName::SETTINGS_ROW)
        .child((row_text(title.into(), note.into()), control))
}

/// A dropdown of a setting.
///
/// # Arguments
///
/// * `name` - Accessible name of the dropdown.
/// * `choices` - The choices.
/// * `value` - The value of the picked choice.
/// * `on_pick` - Called with a picked value.
///
/// # Returns
///
/// The builder, for the options that differ.
pub fn picker(
    name: impl Into<String>, choices: Vec<Choice>, value: Signal<String>,
    on_pick: impl Fn(String) + Send + Sync + 'static,
) -> Dropdown
{
    Dropdown::builder()
        .label(name.into())
        .choices(choices)
        .value(value)
        .on_pick(Callback::new(on_pick))
        .build()
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
pub fn switch(
    on: Signal<bool>, off_limits: Signal<bool>, set: impl Into<SignalSetter<bool>>,
) -> impl IntoView
{
    let set: SignalSetter<bool> = set.into();
    let check = input()
        .r#type("checkbox")
        .prop("checked", on)
        .prop("disabled", off_limits)
        .on(ev::change, move |ev| set.set(event_target_checked(&ev)));
    span()
        .class(ClassName::SETTINGS_SWITCH)
        .child((check, span().class(ClassName::SWITCH_TRACK)))
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

/// Picker of a Burmese font.
///
/// # Arguments
///
/// * `name` - Accessible name of the picker.
/// * `face` - The picked face.
/// * `set` - Pick a face.
///
/// # Returns
///
/// The view.
fn burmese_picker(
    name: impl Into<String>, face: Signal<Face>, set: WriteSignal<Face>,
) -> impl IntoView
{
    let current = Signal::derive(move || face.get().to_string());
    let on_pick = move |v: String| {
        do_if!(let Ok(f) = v.parse::<Face>(), set.set(f));
    };
    dropdown(picker(name, face_choices(Face::burmese_faces()), current, on_pick))
}

/// Picker of a fallback font: Inherit first, then the faces.
///
/// # Arguments
///
/// * `name` - Accessible name of the picker.
/// * `faces` - The faces it offers after Inherit.
/// * `fallback` - The picked fallback.
/// * `set` - Pick a fallback.
///
/// # Returns
///
/// The view.
fn fallback_picker(
    name: impl Into<String>, faces: impl Iterator<Item = Face>,
    fallback: Signal<Fallback>, set: WriteSignal<Fallback>,
) -> impl IntoView
{
    let inherit = Choice::builder()
        .value(Fallback::Inherit.to_string())
        .label(t!("web_ui.face.inherit"))
        .build();
    let mut choices = vec![inherit];
    choices.extend(face_choices(faces));
    let current = Signal::derive(move || fallback.get().to_string());
    let on_pick = move |v: String| {
        do_if!(let Ok(f) = v.parse::<Fallback>(), set.set(f));
    };
    dropdown(picker(name, choices, current, on_pick))
}

/// Picker of the code font size.
///
/// # Arguments
///
/// * `look` - The look.
///
/// # Returns
///
/// The view.
fn size_picker(look: Look) -> impl IntoView
{
    let choice = |n: &u32| {
        let label = t!("web_ui.settings.code_size_choice", n = n);
        Choice::builder().value(n.to_string()).label(label).build()
    };
    let current = Signal::derive(move || look.font_size.get().to_string());
    let on_pick =
        move |v: String| look.set_font_size.set(v.parse().unwrap_or_default());
    dropdown(picker(
        t!("web_ui.settings.code_size"),
        FONT_SIZES.iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Switch that draws comments in italics.
///
/// # Arguments
///
/// * `look` - The look.
///
/// # Returns
///
/// The view.
fn italic_switch(look: Look) -> impl IntoView
{
    switch(look.italic_comments, Signal::stored(false), look.set_italic_comments)
}
