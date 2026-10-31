use std::borrow::Cow;

use leptos::prelude::*;
use strum::IntoEnumIterator;
use zaka_vm::machine::FrameMode;
use zaka_web_ui_common::controls::ClassName as Controls;
use zaka_web_ui_common::controls::dropdown::Choice;
use zaka_web_ui_common::controls::dropdown::dropdown;
use zaka_web_ui_common::controls::modal::Modal;
use zaka_web_ui_common::controls::modal::modal;
use zaka_web_ui_common::controls::pickers::lang_picker;
use zaka_web_ui_common::controls::settings::appearance;
use zaka_web_ui_common::controls::settings::picker;
use zaka_web_ui_common::controls::settings::row;
use zaka_web_ui_common::controls::settings::section;
use zaka_web_ui_common::controls::settings::settings_modal as shared_modal;
use zaka_web_ui_common::controls::settings::switch;
use zaka_web_ui_common::controls::settings::switch_row;
use zakac_borrow_checker::options::MemCheck;
use zakac_codegen::options::ResultMode;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;

use crate::compile::Options;
use crate::compile::PLATFORM_BITS;
use crate::state::AppState;
use crate::state::settings::HeapCap;
use crate::state::use_state;

/// Element id of the title of the compiler options modal.
const COMPILER_TITLE_ID: &str = "compiler-modal-title";

/// The settings modal: the language, auto run, the look of the page and the
/// compile options. The toolbar button and the menu of narrow screens both
/// open it.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
///
/// # Returns
///
/// The view.
pub fn settings_modal(open: RwSignal<bool>) -> impl IntoView
{
    let state = use_state();
    let auto_run = switch(state.auto_run, Signal::stored(false), state.set_auto_run);
    let general = section(
        t!("explorer.settings.general"),
        (
            row(
                t!("explorer.settings.language"),
                t!("explorer.settings.language_note"),
                lang_picker(move || state.flush_link()),
            ),
            switch_row(
                t!("explorer.settings.auto_run"),
                t!("explorer.settings.auto_run_note"),
                auto_run,
            ),
        ),
    );
    shared_modal(open, (general, appearance(state.look), compiler_section(state)))
}

/// The compiler options: the platform bits, the foldings of the IR, the
/// heap limit and the frames of a run, and the ownership checks. The checks
/// are not stored on the device; the link carries them.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn compiler_section(state: AppState) -> impl IntoView
{
    section(
        t!("explorer.settings.compiler"),
        (
            row(
                t!("explorer.settings.platform_bits"),
                t!("explorer.settings.platform_bits_note"),
                platform_picker(state),
            ),
            fold_row(state, Fold::Exprs),
            fold_row(state, Fold::Consts),
            fold_row(state, Fold::Branches),
            row(
                t!("explorer.settings.heap_cap"),
                t!("explorer.settings.heap_cap_note"),
                heap_picker(state),
            ),
            row(
                t!("explorer.settings.frames"),
                t!("explorer.settings.frames_note"),
                frames_picker(state),
            ),
            row(
                t!("explorer.settings.mem_check"),
                t!("explorer.settings.mem_check_note"),
                mem_check_picker(state),
            ),
            option_row(
                state,
                (
                    t!("explorer.settings.mem_strict"),
                    t!("explorer.settings.mem_strict_note"),
                ),
                |o| o.strict,
                |o, on| o.strict = on,
            ),
            option_row(
                state,
                (t!("explorer.settings.no_raw"), t!("explorer.settings.no_raw_note")),
                |o| o.no_raw,
                |o, on| o.no_raw = on,
            ),
            row(
                t!("explorer.settings.results"),
                t!("explorer.settings.results_note"),
                results_picker(state),
            ),
        ),
    )
}

/// The row of a compile option with a switch.
///
/// # Arguments
///
/// * `state` - The state.
/// * `text` - The name of the option and the note under it.
/// * `read` - Whether the option is on.
/// * `write` - Turn the option on or off.
///
/// # Returns
///
/// The view.
fn option_row(
    state: AppState, text: (Cow<'static, str>, Cow<'static, str>),
    read: fn(&Options) -> bool, write: fn(&mut Options, bool),
) -> impl IntoView
{
    let on = Signal::derive(move || state.options.with(read));
    let set = SignalSetter::map(move |b| state.options.update(|o| write(o, b)));
    switch_row(text.0, text.1, switch(on, Signal::stored(false), set))
}

/// The modal of the compiler options, which the debugger bar opens; the
/// settings hold the same options.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
///
/// # Returns
///
/// The view.
pub fn compiler_modal(open: RwSignal<bool>) -> impl IntoView
{
    let state = use_state();
    let spec = Modal::builder()
        .open(open)
        .class(Controls::SETTINGS_MODAL)
        .title_id(COMPILER_TITLE_ID)
        .title(t!("explorer.run.options"))
        .close_label(t!("explorer.run.options_close"))
        .build();
    modal(spec, compiler_section(state))
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

    /// The name of the setting and the note under it.
    ///
    /// # Returns
    ///
    /// The two texts.
    fn text(self) -> (String, String)
    {
        let (name, note) = match self
        {
            Fold::Exprs => (
                t!("explorer.settings.fold_exprs"),
                t!("explorer.settings.fold_exprs_note"),
            ),
            Fold::Consts => (
                t!("explorer.settings.fold_consts"),
                t!("explorer.settings.fold_consts_note"),
            ),
            Fold::Branches => (
                t!("explorer.settings.fold_branches"),
                t!("explorer.settings.fold_branches_note"),
            ),
        };
        (name.into(), note.into())
    }

    /// Whether the folding is on.
    ///
    /// # Arguments
    ///
    /// * `state` - The state.
    ///
    /// # Returns
    ///
    /// The switch.
    fn on(self, state: AppState) -> Signal<bool>
    {
        Signal::derive(move || *self.flag(&mut state.options.get().fold))
    }

    /// Turn the folding on or off.
    ///
    /// # Arguments
    ///
    /// * `state` - The state.
    /// * `on` - Whether it is on.
    fn set(self, state: AppState, on: bool)
    {
        state.options.update(|o| *self.flag(&mut o.fold) = on);
    }
}

/// The row of one folding of the IR, with its switch.
///
/// # Arguments
///
/// * `state` - The state.
/// * `fold` - The folding.
///
/// # Returns
///
/// The view.
fn fold_row(state: AppState, fold: Fold) -> impl IntoView
{
    let on = fold.on(state);
    let set = SignalSetter::map(move |b| fold.set(state, b));
    let (name, note) = fold.text();
    switch_row(name, note, switch(on, Signal::stored(false), set))
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
        t!("explorer.settings.platform_bits"),
        PLATFORM_BITS.iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of how the variables of a run sit in the address space.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn frames_picker(state: AppState) -> impl IntoView
{
    let label = |f: FrameMode| match f
    {
        FrameMode::Checked => t!("explorer.settings.frames_checked"),
        FrameMode::Packed => t!("explorer.settings.frames_packed"),
    };
    let choice = move |f: FrameMode| {
        Choice::builder().value(f.to_string()).label(label(f)).build()
    };
    let current = Signal::derive(move || state.frames.get().to_string());
    let on_pick = move |v: String| state.set_frames.set(v.parse().unwrap_or_default());
    dropdown(picker(
        t!("explorer.settings.frames"),
        FrameMode::iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of what a pointer mistake the ownership check proves does.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn mem_check_picker(state: AppState) -> impl IntoView
{
    let label = |m: MemCheck| match m
    {
        MemCheck::Error => t!("explorer.settings.mem_check_error"),
        MemCheck::Warn => t!("explorer.settings.mem_check_warn"),
        MemCheck::Off => t!("explorer.settings.mem_check_off"),
    };
    let choice = move |m: MemCheck| {
        Choice::builder().value(m.to_string()).label(label(m)).build()
    };
    let current = Signal::derive(move || state.options.get().check.to_string());
    let on_pick = move |v: String| {
        let check = v.parse().unwrap_or_default();
        state.options.update(|o| o.check = check);
    };
    dropdown(picker(
        t!("explorer.settings.mem_check"),
        MemCheck::iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of how a list or struct result reaches the caller.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn results_picker(state: AppState) -> impl IntoView
{
    let label = |m: ResultMode| match m
    {
        ResultMode::InPlace => t!("explorer.settings.results_in_place"),
        ResultMode::Duplicate => t!("explorer.settings.results_duplicate"),
    };
    let choice = move |m: ResultMode| {
        Choice::builder().value(m.to_string()).label(label(m)).build()
    };
    let current = Signal::derive(move || state.options.get().results.to_string());
    let on_pick = move |v: String| {
        let results = v.parse().unwrap_or_default();
        state.options.update(|o| o.results = results);
    };
    dropdown(picker(
        t!("explorer.settings.results"),
        ResultMode::iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of the cap on live heap octets of a run.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn heap_picker(state: AppState) -> impl IntoView
{
    let choice =
        |c: HeapCap| Choice::builder().value(c.to_string()).label(c.label()).build();
    let current = Signal::derive(move || state.heap_cap.get().to_string());
    let on_pick =
        move |v: String| state.set_heap_cap.set(v.parse().unwrap_or_default());
    dropdown(picker(
        t!("explorer.settings.heap_cap"),
        HeapCap::iter().map(choice).collect(),
        current,
        on_pick,
    ))
}
