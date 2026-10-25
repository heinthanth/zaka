use leptos::prelude::*;
use zaka_web_ui_common::controls::dropdown::Choice;
use zaka_web_ui_common::controls::dropdown::dropdown;
use zaka_web_ui_common::controls::pickers::lang_picker;
use zaka_web_ui_common::controls::settings::appearance;
use zaka_web_ui_common::controls::settings::picker;
use zaka_web_ui_common::controls::settings::row;
use zaka_web_ui_common::controls::settings::section;
use zaka_web_ui_common::controls::settings::settings_menu as shared_menu;
use zaka_web_ui_common::controls::settings::switch;
use zaka_web_ui_common::controls::settings::switch_row;
use zaka_web_ui_common::font::face::Face;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;

use crate::compile::PLATFORM_BITS;
use crate::font::HAS_CODE_FONT;
use crate::state::AppState;
use crate::state::use_state;

/// The settings button and the modal it opens: the language, the look of the
/// page and the compile options.
///
/// # Returns
///
/// The view.
pub fn settings_menu() -> impl IntoView
{
    let state = use_state();
    let general = section(
        t!("explorer.settings.general"),
        row(
            t!("explorer.settings.language"),
            t!("explorer.settings.language_note"),
            lang_picker(move || state.flush_link()),
        ),
    );
    let pda = HAS_CODE_FONT.then(|| {
        switch_row(
            t!("explorer.settings.pda"),
            t!("explorer.settings.pda_note"),
            pda_switch(state),
        )
        .into_any()
    });
    let compiler = section(
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
        ),
    );
    shared_menu((general, appearance(state.look, pda), compiler))
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
    let look = state.look;
    let is_pda = Signal::derive(move || look.code_font.get() == Face::Pda10);
    let on = Signal::derive(move || look.pda_burmese.get() && !is_pda.get());
    switch(on, is_pda, look.set_pda_burmese)
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
    let on = Signal::derive(move || *fold.flag(&mut state.options.get().fold));
    let set = SignalSetter::map(move |b| {
        state.options.update(|o| *fold.flag(&mut o.fold) = b)
    });
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
