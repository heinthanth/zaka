use std::borrow::Cow;

use leptos::ev;
use leptos::html::button;
use leptos::prelude::*;
use strum::IntoEnumIterator;
use strum::VariantNames;
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
use zaka_web_ui_common::icon::chevron;
use zaka_web_ui_common::icon::sliders;
use zakac_ast::target::TargetAbi;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_borrow_checker::options::MemCheck;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::options::RunCheck;
use zakac_codegen_target_c::options::RunChecks;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_codegen_target_vm::options::ScratchMode;
use zakac_diagnostics::Code;
use zakac_diagnostics::lint::LINT_CODES;
use zakac_diagnostics::lint::LintGroup;
use zakac_diagnostics::lint::LintLevel;
use zakac_diagnostics::lint::LintLevels;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;

use crate::compile::Options;
use crate::compile::PLATFORM_BITS;
use crate::compile::Target;
use crate::state::AppState;
use crate::state::settings::HeapCap;
use crate::state::use_state;

/// Element id of the title of the compiler options modal.
const COMPILER_TITLE_ID: &str = "compiler-modal-title";

/// The settings modal: the language, auto run, the look of the page and the
/// compile options. The toolbar button and the menu of narrow screens both
/// open it, or the header of the website page. There the header picks the
/// language and the theme, so the modal leaves them out.
///
/// # Arguments
///
/// * `open` - Whether the modal shows.
/// * `on_site` - Whether the explorer sits in a page of the website.
///
/// # Returns
///
/// The view.
pub fn settings_modal(open: RwSignal<bool>, on_site: bool) -> impl IntoView
{
    let state = use_state();
    let auto_run = switch(state.auto_run, Signal::stored(false), state.set_auto_run);
    let general = section(
        t!("explorer.settings.general"),
        (
            (!on_site).then(|| {
                row(
                    t!("explorer.settings.language"),
                    t!("explorer.settings.language_note"),
                    lang_picker(move || state.flush_link()),
                )
            }),
            switch_row(
                t!("explorer.settings.auto_run"),
                t!("explorer.settings.auto_run_note"),
                auto_run,
            ),
        ),
    );
    shared_modal(
        open,
        (general, appearance(state.look, !on_site), compiler_section(state)),
    )
}

/// The compiler options: the target and, for C, its run-time checks, the
/// target OS and CPU, the platform bits, the foldings of the IR, the heap
/// limit and the frames of a run, and the checkers. The lint levels open on a
/// second page in place of the rows. The checks are not stored on the device;
/// the link carries them.
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
    let lints_open = RwSignal::new(false);
    let rows = move || {
        tern!(
            lints_open.get(),
            lint_page(state, lints_open).into_any(),
            compiler_rows(state, lints_open).into_any()
        )
    };
    section(t!("explorer.settings.compiler"), rows)
}

/// The rows of the compiler options.
///
/// # Arguments
///
/// * `state` - The state.
/// * `lints_open` - Whether the lint page shows.
///
/// # Returns
///
/// The view.
fn compiler_rows(state: AppState, lints_open: RwSignal<bool>) -> impl IntoView
{
    let c_mode = move || state.c_mode.get();
    let native = move || state.options.get().target == Target::Native;
    (
        row(
            t!("explorer.settings.target"),
            t!("explorer.settings.target_note"),
            target_picker(state),
        ),
        move || {
            c_mode().then(|| {
                (
                    row(
                        t!("explorer.settings.c_checks"),
                        t!("explorer.settings.c_checks_note"),
                        c_checks_picker(state),
                    ),
                    RunCheck::iter().map(|c| c_check_row(state, c)).collect_view(),
                )
            })
        },
        row(
            t!("explorer.settings.target_os"),
            t!("explorer.settings.target_os_note"),
            os_picker(state),
        ),
        move || {
            (!native()).then(|| {
                row(
                    t!("explorer.settings.target_arch"),
                    t!("explorer.settings.target_arch_note"),
                    arch_picker(state),
                )
            })
        },
        row(
            t!("explorer.settings.target_abi"),
            t!("explorer.settings.target_abi_note"),
            abi_picker(state),
        ),
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
        (
            row(
                t!("explorer.settings.mem_check"),
                t!("explorer.settings.mem_check_note"),
                mem_check_picker(state),
            ),
            row(
                t!("explorer.settings.lints"),
                t!("explorer.settings.lints_note"),
                page_button(t!("explorer.settings.lints_open"), sliders(), move || {
                    lints_open.set(true)
                }),
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
                (
                    t!("explorer.settings.no_unsafe"),
                    t!("explorer.settings.no_unsafe_note"),
                ),
                |o| o.no_unsafe,
                |o, on| o.no_unsafe = on,
            ),
            option_row(
                state,
                (
                    t!("explorer.settings.overflow"),
                    t!("explorer.settings.overflow_note"),
                ),
                |o| o.overflow == OverflowMode::Stop,
                |o, on| o.overflow = tern!(on, OverflowMode::Stop, OverflowMode::Wrap),
            ),
            row(
                t!("explorer.settings.results"),
                t!("explorer.settings.results_note"),
                results_picker(state),
            ),
            row(
                t!("explorer.settings.scratch"),
                t!("explorer.settings.scratch_note"),
                scratch_picker(state),
            ),
        ),
    )
}

/// A square button with an icon that opens or leaves a page of the settings.
///
/// # Arguments
///
/// * `name` - Its accessible name and its tooltip.
/// * `icon` - The icon.
/// * `on_click` - What a click does.
///
/// # Returns
///
/// The view.
fn page_button(
    name: Cow<'static, str>, icon: impl IntoView + 'static,
    on_click: impl Fn() + 'static,
) -> impl IntoView
{
    button()
        .class(Controls::ICON_BUTTON)
        .attr("aria-label", name.to_string())
        .title(name.to_string())
        .on(ev::click, move |_| on_click())
        .child(icon)
}

/// The lint page: a level for each group of lint codes, then for each code,
/// with the short title of the code. A level set here holds over the level
/// of the code, as `--lint-allow`, `--lint-warn` and `--lint-error` do.
///
/// # Arguments
///
/// * `state` - The state.
/// * `lints_open` - Whether the lint page shows.
///
/// # Returns
///
/// The view.
fn lint_page(state: AppState, lints_open: RwSignal<bool>) -> impl IntoView
{
    let back = row(
        t!("explorer.settings.lints"),
        t!("explorer.settings.lints_page_note"),
        page_button(t!("explorer.settings.lints_back"), chevron(), move || {
            lints_open.set(false)
        }),
    );
    let groups = LintGroup::iter().map(|g| group_row(state, g)).collect_view();
    let codes = LINT_CODES.iter().map(|c| code_row(state, *c)).collect_view();
    (back, groups, codes)
}

/// The word of a lint level in the lint pickers; `default` for none.
///
/// # Arguments
///
/// * `level` - The level, or `None` for the level of the code.
///
/// # Returns
///
/// The word.
fn level_word(level: Option<LintLevel>) -> String
{
    level.map_or_else(|| DEFAULT_LEVEL.to_string(), |l| l.to_string())
}

/// The word of the level of the code itself in the lint pickers.
const DEFAULT_LEVEL: &str = "default";

/// The choices of a lint picker: the level of the code, then off, warn and
/// error.
///
/// # Returns
///
/// The choices.
fn level_choices() -> Vec<Choice>
{
    let label = |l: Option<LintLevel>| match l
    {
        None => t!("explorer.settings.lint_default"),
        Some(LintLevel::Off) => t!("explorer.settings.lint_off"),
        Some(LintLevel::Warn) => t!("explorer.settings.lint_warn"),
        Some(LintLevel::Error) => t!("explorer.settings.lint_error"),
    };
    let levels = std::iter::once(None).chain(LintLevel::iter().map(Some));
    let choice = |l| Choice::builder().value(level_word(l)).label(label(l)).build();
    levels.map(choice).collect()
}

/// The row of a group of lint codes: one level for each code of the group.
///
/// # Arguments
///
/// * `state` - The state.
/// * `group` - The group.
///
/// # Returns
///
/// The view.
fn group_row(state: AppState, group: LintGroup) -> impl IntoView
{
    let name: &'static str = group.into();
    // The group shows a level when each of its codes holds it, and none else.
    let current = Signal::derive(move || {
        let lint = state.options.get().lint;
        let mut levels = group.codes().iter().map(|c| lint.get(*c));
        let first = levels.next().flatten();
        let same = levels.all(|l| l == first);
        tern!(same, level_word(first), String::new())
    });
    let on_pick = move |v: String| {
        let level = v.parse::<LintLevel>().ok();
        let set =
            |lint: LintLevels| group.codes().iter().fold(lint, |l, c| l.set(*c, level));
        state.options.update(|o| o.lint = set(o.lint));
    };
    let note = group_note(group);
    row(name, note, dropdown(picker(name, level_choices(), current, on_pick)))
}

/// The row of one lint code: its id, its short title and its level.
///
/// # Arguments
///
/// * `state` - The state.
/// * `code` - The code.
///
/// # Returns
///
/// The view.
fn code_row(state: AppState, code: Code) -> impl IntoView
{
    let current =
        Signal::derive(move || level_word(state.options.get().lint.get(code)));
    let on_pick = move |v: String| {
        let level = v.parse::<LintLevel>().ok();
        state.options.update(|o| o.lint = o.lint.set(code, level));
    };
    let picker = picker(code.id(), level_choices(), current, on_pick);
    row(code.id(), code.title(), dropdown(picker))
}

/// The row of one run-time check of the C files, with a switch.
///
/// # Arguments
///
/// * `state` - The state.
/// * `check` - The check.
///
/// # Returns
///
/// The view.
fn c_check_row(state: AppState, check: RunCheck) -> impl IntoView
{
    let name: &'static str = check.into();
    let on = Signal::derive(move || state.options.get().c_checks.has(check));
    let set = SignalSetter::map(move |b| {
        state.options.update(|o| o.c_checks = o.c_checks.with(check, b))
    });
    let note = check_note(check);
    switch_row(format!("+{name}"), note, switch(on, Signal::stored(false), set))
}

/// The note of a lint group.
///
/// # Arguments
///
/// * `group` - The group.
///
/// # Returns
///
/// The note.
fn group_note(group: LintGroup) -> Cow<'static, str>
{
    match group
    {
        LintGroup::Mem => t!("explorer.lint_group.mem"),
        LintGroup::Leaks => t!("explorer.lint_group.leaks"),
        LintGroup::Frees => t!("explorer.lint_group.frees"),
        LintGroup::Borrows => t!("explorer.lint_group.borrows"),
        LintGroup::Nulls => t!("explorer.lint_group.nulls"),
        LintGroup::Pointers => t!("explorer.lint_group.pointers"),
        LintGroup::Doubts => t!("explorer.lint_group.doubts"),
        LintGroup::Overflow => t!("explorer.lint_group.overflow"),
        LintGroup::Naturalness => t!("explorer.lint_group.naturalness"),
        LintGroup::Spoken => t!("explorer.lint_group.spoken"),
    }
}

/// The note of a run-time check of the C files.
///
/// # Arguments
///
/// * `check` - The check.
///
/// # Returns
///
/// The note.
fn check_note(check: RunCheck) -> Cow<'static, str>
{
    match check
    {
        RunCheck::Divide => t!("explorer.c_check.divide"),
        RunCheck::Shift => t!("explorer.c_check.shift"),
        RunCheck::Bounds => t!("explorer.c_check.bounds"),
        RunCheck::Text => t!("explorer.c_check.text"),
        RunCheck::Step => t!("explorer.c_check.step"),
        RunCheck::Depth => t!("explorer.c_check.depth"),
        RunCheck::Null => t!("explorer.c_check.null"),
        RunCheck::Overlap => t!("explorer.c_check.overlap"),
        RunCheck::Sanitizers => t!("explorer.c_check.sanitizers"),
    }
}

/// The name of an OS as the picker shows it.
///
/// # Arguments
///
/// * `os` - The OS.
///
/// # Returns
///
/// The name.
fn os_label(os: TargetOs) -> Cow<'static, str>
{
    match os
    {
        TargetOs::Windows => t!("explorer.settings.os_windows"),
        TargetOs::Macos => t!("explorer.settings.os_macos"),
        TargetOs::Linux => t!("explorer.settings.os_linux"),
        TargetOs::Wasi => t!("explorer.settings.os_wasi"),
    }
}

/// Picker of the OS the compile-time conditionals see.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn os_picker(state: AppState) -> impl IntoView
{
    let choice = |os: TargetOs| {
        let name: &'static str = os.into();
        Choice::builder().value(name).label(os_label(os)).build()
    };
    let current = Signal::derive(move || {
        <&'static str>::from(state.options.get().os).to_string()
    });
    let on_pick = move |v: String| {
        let Ok(os) = v.parse::<TargetOs>()
        else
        {
            return;
        };
        // WASI runs 32-bit WebAssembly alone, as `--target-os wasi` takes it.
        let wasi = os == TargetOs::Wasi;
        state.options.update(|o| {
            o.os = os;
            do_if!(wasi, (o.arch, o.platform_bits) = (TargetArch::Wasm32, 32));
        });
    };
    dropdown(picker(
        t!("explorer.settings.target_os"),
        TargetOs::VARIANTS.iter().filter_map(|v| v.parse().ok()).map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of the CPU the compile-time conditionals see.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn arch_picker(state: AppState) -> impl IntoView
{
    let choice = |name: &&'static str| {
        Choice::builder().value(name.to_string()).label(name.to_string()).build()
    };
    let current = Signal::derive(move || {
        <&'static str>::from(state.options.get().arch).to_string()
    });
    let on_pick = move |v: String| {
        let Ok(arch) = v.parse::<TargetArch>()
        else
        {
            return;
        };
        state.options.update(|o| o.arch = arch);
    };
    dropdown(picker(
        t!("explorer.settings.target_arch"),
        TargetArch::VARIANTS.iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of the ABI the compile-time conditionals see: the usual one of
/// the OS, or one of the ABIs by name.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn abi_picker(state: AppState) -> impl IntoView
{
    const USUAL: &str = "usual";
    let choice = |value: &str, label: Cow<'static, str>| {
        Choice::builder().value(value.to_string()).label(label).build()
    };
    let usual = choice(USUAL, t!("explorer.settings.abi_usual"));
    let named = TargetAbi::VARIANTS.iter().map(|n| choice(n, Cow::Borrowed(*n)));
    let current = Signal::derive(move || {
        let abi = state.options.get().abi;
        abi.map_or(USUAL, <&'static str>::from).to_string()
    });
    let on_pick = move |v: String| state.options.update(|o| o.abi = v.parse().ok());
    dropdown(picker(
        t!("explorer.settings.target_abi"),
        std::iter::once(usual).chain(named).collect(),
        current,
        on_pick,
    ))
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

/// Picker of what a pointer mistake the checker proves does.
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

/// Picker of what code generation writes: ops for the VM, or C.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn target_picker(state: AppState) -> impl IntoView
{
    let label = |m: Target| match m
    {
        Target::Vm => t!("explorer.settings.target_vm"),
        Target::C => t!("explorer.settings.target_c"),
        Target::Native => t!("explorer.settings.target_native"),
    };
    let choice =
        move |m: Target| Choice::builder().value(m.to_string()).label(label(m)).build();
    let current = Signal::derive(move || state.options.get().target.to_string());
    let on_pick = move |v: String| {
        let target = v.parse().unwrap_or_default();
        state.options.update(|o| o.target = target);
    };
    dropdown(picker(
        t!("explorer.settings.target"),
        Target::iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of the run-time checks the C files keep.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn c_checks_picker(state: AppState) -> impl IntoView
{
    let label = |c: Checks| match c
    {
        Checks::None => t!("explorer.settings.c_checks_none"),
        Checks::Default => t!("explorer.settings.c_checks_default"),
        Checks::Mem => t!("explorer.settings.c_checks_mem"),
    };
    let choice =
        move |c: Checks| Choice::builder().value(c.to_string()).label(label(c)).build();
    // The level shows when the checks are those of a level, and none else.
    let current = Signal::derive(move || {
        let checks = state.options.get().c_checks;
        let level = Checks::iter().find(|l| RunChecks::from(*l) == checks);
        level.map(|l| l.to_string()).unwrap_or_default()
    });
    let on_pick = move |v: String| {
        let level = v.parse::<Checks>().unwrap_or_default();
        state.options.update(|o| o.c_checks = level.into());
    };
    dropdown(picker(
        t!("explorer.settings.c_checks"),
        Checks::iter().map(choice).collect(),
        current,
        on_pick,
    ))
}

/// Picker of where the scratch values of a function live.
///
/// # Arguments
///
/// * `state` - The state.
///
/// # Returns
///
/// The view.
fn scratch_picker(state: AppState) -> impl IntoView
{
    let label = |m: ScratchMode| match m
    {
        ScratchMode::Apart => t!("explorer.settings.scratch_apart"),
        ScratchMode::Shared => t!("explorer.settings.scratch_shared"),
    };
    let choice = move |m: ScratchMode| {
        Choice::builder().value(m.to_string()).label(label(m)).build()
    };
    let current = Signal::derive(move || state.options.get().scratch.to_string());
    let on_pick = move |v: String| {
        let scratch = v.parse().unwrap_or_default();
        state.options.update(|o| o.scratch = scratch);
    };
    dropdown(picker(
        t!("explorer.settings.scratch"),
        ScratchMode::iter().map(choice).collect(),
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
