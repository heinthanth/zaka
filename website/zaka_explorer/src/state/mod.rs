use std::collections::BTreeSet;

use bon::Builder;
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use_signal_debounce_throttle::signal_debounced;
use leptos_use_storage::UseStorageOptions;
use leptos_use_storage::use_local_storage;
use leptos_use_storage::use_local_storage_with_options;
use zaka_web_ui_common::font::load_pair;
pub use zaka_web_ui_common::lang;
use zaka_web_ui_common::look::Look;
use zaka_web_ui_common::look::provide_look;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
pub mod permalink;
pub mod settings;
pub mod split;
pub mod tour;
pub mod url;

use crate::compile::Compiled;
use crate::compile::Options;
use crate::compile::Phase;
use crate::compile::asm::OpSpot;
use crate::compile::asm::find_spot;
use crate::compile::asm::has_ops;
use crate::compile::asm::line_stops;
use crate::compile::asm::op_spot;
use crate::compile::compile;
use crate::compile::samples::default_sample;
use crate::compile::spans::Range;
use crate::debug::Session;
use crate::debug::Speed;
use crate::monaco::Editor;
use crate::state::permalink::Link;
use crate::state::settings::AstMode;
use crate::state::settings::HeapCap;
use crate::state::settings::LowerMode;
use crate::state::split::DEFAULT_DOCK_SHARE;
use crate::state::split::DEFAULT_EDITOR_SHARE;
use crate::state::split::DEFAULT_STAGE_SHARE;

/// Milliseconds of quiet after an edit before the phases run again.
const SETTLE_MS: f64 = 150.0;

/// Local storage name of the AST view mode.
const AST_MODE_STORE: &str = "zaka-explorer-ast-mode";

/// Local storage name of the lowering view mode.
const LOWER_MODE_STORE: &str = "zaka-explorer-lower-mode";

/// Local storage name of the editor share of the width.
const EDITOR_SHARE_STORE: &str = "zaka-explorer-editor-share";

/// Local storage name of the stage view share of the right column.
const STAGE_SHARE_STORE: &str = "zaka-explorer-stage-share";

/// Local storage name of the stage view share of the debug row.
const DOCK_SHARE_STORE: &str = "zaka-explorer-dock-share";

/// Local storage name of the editor pane switch.
const EDITOR_STORE: &str = "zaka-explorer-editor-open";

/// Local storage name of the heap cap.
const HEAP_CAP_STORE: &str = "zaka-explorer-heap-cap";

/// Local storage name of the stage view switch.
const STAGE_STORE: &str = "zaka-explorer-stage-open";

/// Local storage name of the dock switch.
const DOCK_STORE: &str = "zaka-explorer-dock-open";

/// Local storage name of the problems panel switch.
const PROBLEMS_STORE: &str = "zaka-explorer-problems-open";

/// Local storage name of the debug mode switch.
const DEBUG_STORE: &str = "zaka-explorer-debug-mode";

/// Local storage name of the auto run switch.
const AUTO_RUN_STORE: &str = "zaka-explorer-auto-run";

/// The tab of the bottom panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelTab
{
    /// What the program wrote; in debug mode alone.
    #[default]
    Output,
    /// The diagnostics.
    Problems,
}

/// State shared between the editor and the views.
#[derive(Clone, Copy, Builder)]
pub struct AppState
{
    /// The text in the editor.
    pub source: RwSignal<String>,
    /// The compile options.
    pub options: RwSignal<Options>,
    /// The phases over the settled text.
    pub compiled: Memo<Compiled>,
    /// The range a view points at; the editor highlights it.
    pub pointer: RwSignal<Option<Range>>,
    /// The token under the caret.
    pub caret_token: RwSignal<Option<usize>>,
    /// The token under the mouse in the editor.
    pub mouse_token: RwSignal<Option<usize>>,
    /// The open stage of the pipeline.
    pub stage: RwSignal<Phase>,
    /// The debugger session over the generated ops.
    pub debug: RwSignal<Session>,
    /// The editor, once Monaco has loaded.
    pub editor: StoredValue<Option<Editor>, LocalStorage>,
    /// Whether the editor has loaded.
    pub mounted: RwSignal<bool>,
    /// The theme, the fonts and the code size, shared with the website.
    pub look: Look,
    /// How the AST view shows the tree.
    pub ast_mode: Signal<AstMode>,
    /// Choose how the AST view shows the tree.
    pub set_ast_mode: WriteSignal<AstMode>,
    /// Which IR the lowering view shows.
    pub lower_mode: Signal<LowerMode>,
    /// Choose which IR the lowering view shows.
    pub set_lower_mode: WriteSignal<LowerMode>,
    /// Share of the width the editor takes.
    pub editor_share: Signal<f64>,
    /// Set the editor share of the width.
    pub set_editor_share: WriteSignal<f64>,
    /// Share of the right column the stage view takes.
    pub stage_share: Signal<f64>,
    /// Set the stage view share of the right column.
    pub set_stage_share: WriteSignal<f64>,
    /// Share of the debug row the stage view takes; the dock takes the rest.
    pub dock_share: Signal<f64>,
    /// Set the stage view share of the debug row.
    pub set_dock_share: WriteSignal<f64>,
    /// Whether the editor pane shows.
    pub editor_open: Signal<bool>,
    /// Show or hide the editor pane.
    pub set_editor_open: WriteSignal<bool>,
    /// Whether the stage view shows.
    pub stage_open: Signal<bool>,
    /// Show or hide the stage view.
    pub set_stage_open: WriteSignal<bool>,
    /// Whether the dock shows in debug mode.
    pub dock_open: Signal<bool>,
    /// Show or hide the dock.
    pub set_dock_open: WriteSignal<bool>,
    /// Whether the problems panel is open.
    pub problems_open: Signal<bool>,
    /// Open or close the problems panel.
    pub set_problems_open: WriteSignal<bool>,
    /// Whether debug mode is on: the pipeline shows each phase and the run
    /// view is a debugger. Off, the pane shows the run alone.
    pub debug_mode: Signal<bool>,
    /// Turn debug mode on or off.
    pub set_debug_mode: WriteSignal<bool>,
    /// Whether the debugger steps on a timer.
    pub auto_step: RwSignal<bool>,
    /// The cap on live heap octets of a run.
    pub heap_cap: Signal<HeapCap>,
    /// Choose the cap on live heap octets.
    pub set_heap_cap: WriteSignal<HeapCap>,
    /// Whether ops run in batches, one batch a frame.
    pub going: RwSignal<bool>,
    /// The speed of the timed stepping.
    pub speed: RwSignal<Speed>,
    /// The open tab of the bottom panel.
    pub panel_tab: RwSignal<PanelTab>,
    /// The address a click on a pointer asks the heap panel to show.
    pub heap_focus: RwSignal<Option<u64>>,
    /// The 1-based source lines with a break point. The editor moves them
    /// with the edits; each compile turns them into stops of the debugger.
    pub break_lines: RwSignal<BTreeSet<u32>>,
    /// The break points set on ops, keyed on where each op comes from in the
    /// source; a change of the compile options keeps them, an edit of the
    /// text clears them.
    pub op_breaks: RwSignal<BTreeSet<OpSpot>>,
    /// Whether the program runs on its own each time the code compiles,
    /// outside debug mode. Off, go starts each run.
    pub auto_run: Signal<bool>,
    /// Turn auto run on or off.
    pub set_auto_run: WriteSignal<bool>,
    /// Whether the guided tour is open.
    pub tour: RwSignal<bool>,
}

impl AppState
{
    /// The token the editor points at: under the mouse, else under the caret.
    ///
    /// # Returns
    ///
    /// The token index.
    pub fn focus_token(&self) -> Option<usize>
    {
        self.mouse_token.get().or_else(|| self.caret_token.get())
    }

    /// The token at a UTF-16 offset of the editor.
    ///
    /// # Arguments
    ///
    /// * `off16` - The offset.
    ///
    /// # Returns
    ///
    /// The token index, or `None`.
    pub fn token_at(&self, off16: u32) -> Option<usize>
    {
        self.compiled.with_untracked(|c| c.token_at(off16))
    }

    /// Whether a dark theme is on.
    ///
    /// # Returns
    ///
    /// `true` for Dark and the dark Catppuccin flavours.
    pub fn is_dark(&self) -> bool { self.look.is_dark() }

    /// The CSS font stack and size of code text, without tracking.
    ///
    /// # Returns
    ///
    /// The stack and the size in px.
    pub fn font_untracked(&self) -> (String, u32) { self.look.font_untracked() }

    /// The CSS font stack of code text.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn code_stack(&self) -> String { self.look.code_stack() }

    /// Run `f` with the editor, when it has loaded.
    ///
    /// # Arguments
    ///
    /// * `f` - The function.
    pub fn with_editor(&self, f: impl FnOnce(&Editor))
    {
        self.editor.with_value(|editor| editor.as_ref().map(f));
    }

    /// Replace the source text.
    ///
    /// # Arguments
    ///
    /// * `text` - The new text.
    pub fn load(&self, text: String)
    {
        let fresh = self.source.with_untracked(|s| *s != text);
        do_if!(fresh, self.clear_breaks());
        self.with_editor(|editor| editor.set_text(&text));
        self.source.set(text);
    }

    /// Set or clear the break point on a source line; in debug mode alone,
    /// where the gutter for them shows.
    ///
    /// # Arguments
    ///
    /// * `line` - The 1-based line.
    pub fn toggle_line(&self, line: u32)
    {
        ret_if!(!self.debug_mode.get_untracked());
        self.break_lines.update(|lines| {
            tern!(lines.remove(&line), (), _ = lines.insert(line));
        });
    }

    /// Set or clear the break point on an op. A stop of a break point on a
    /// source line clears that line, so the editor and the ops agree.
    ///
    /// # Arguments
    ///
    /// * `func` - The function index.
    /// * `pc` - The op index.
    pub fn toggle_op(&self, func: u32, pc: u32)
    {
        let from_line = self.debug.with_untracked(|s| s.from_line(func, pc));
        let (spot, line) = self
            .compiled
            .with_untracked(|c| {
                let line = c.asm.get(func as usize)?.rows.get(pc as usize)?.line;
                Some((op_spot(&c.asm, func, pc)?, line))
            })
            .unzip();
        let line = line.flatten().filter(|_| from_line);
        ret_if!(let Some(l) = line, self.toggle_line(l as u32));
        let Some(spot) = spot
        else
        {
            return;
        };
        self.op_breaks.update(|spots| {
            tern!(spots.remove(&spot), (), _ = spots.insert(spot));
        });
    }

    /// Follow an edit of the text: take the lines of the break points from
    /// the editor, which moves them with the edits, two that land on one line
    /// becoming one; and drop the break points on ops, whose ops the edit can
    /// have changed.
    pub fn read_break_lines(&self)
    {
        do_if!(
            self.op_breaks.with_untracked(|s| !s.is_empty()),
            self.op_breaks.set(BTreeSet::new())
        );
        let mut moved = BTreeSet::new();
        self.with_editor(|editor| moved.extend(editor.break_lines()));
        let changed = self.break_lines.with_untracked(|lines| *lines != moved);
        do_if!(changed, self.break_lines.set(moved));
    }

    /// Drop all break points on source lines, from the editor first, so the
    /// new text does not keep them.
    fn clear_breaks(&self)
    {
        self.with_editor(|editor| editor.show_breaks(&[]));
        self.break_lines.set(BTreeSet::new());
        self.op_breaks.set(BTreeSet::new());
    }

    /// The ops of the break points on ops in the current ops; one whose op
    /// is gone, as after a change of the folding, stops nothing.
    ///
    /// # Returns
    ///
    /// Function and op index pairs.
    fn op_stops(&self) -> Vec<(u32, u32)>
    {
        self.op_breaks.with(|spots| {
            self.compiled
                .with(|c| spots.iter().filter_map(|s| find_spot(&c.asm, s)).collect())
        })
    }

    /// The source lines of the break points on ops, as the editor marks
    /// them.
    ///
    /// # Returns
    ///
    /// The 1-based lines.
    fn op_lines(&self) -> BTreeSet<u32>
    {
        let line = |c: &Compiled, (f, pc): (u32, u32)| {
            c.asm.get(f as usize)?.rows.get(pc as usize)?.line.map(|l| l as u32)
        };
        let stops = self.op_stops();
        self.compiled.with(|c| stops.into_iter().filter_map(|s| line(c, s)).collect())
    }

    /// The stops of the break points on source lines in the current ops.
    ///
    /// # Returns
    ///
    /// Function and op index pairs.
    fn line_stops(&self) -> Vec<(u32, u32)>
    {
        let stops = |c: &Compiled, l: &u32| line_stops(&c.asm, *l as usize);
        self.break_lines.with(|lines| {
            self.compiled.with(|c| lines.iter().flat_map(|l| stops(c, l)).collect())
        })
    }

    /// Select a range in the editor.
    ///
    /// # Arguments
    ///
    /// * `range` - The range of the original text.
    pub fn select(&self, range: Range)
    {
        let (lo, hi) = self.compiled.with_untracked(|c| c.table.range_to_utf16(&range));
        self.with_editor(|editor| editor.select(lo, hi));
    }

    /// The link of the current text, options, stage and views, without
    /// tracking.
    ///
    /// # Returns
    ///
    /// The link.
    pub fn link(&self) -> Link
    {
        Link::builder()
            .source(self.source.get_untracked())
            .options(self.options.get_untracked())
            .stage(self.stage.get_untracked())
            .ast_mode(self.ast_mode.get_untracked())
            .lower_mode(self.lower_mode.get_untracked())
            .build()
    }

    /// Write the link of the current text to the URL at once, ahead of the
    /// debounce, so a reload right after keeps the latest edits.
    pub fn flush_link(&self) { url::write_hash(&permalink::encode(&self.link())) }

    /// Open a link: its source, options, stage and views.
    ///
    /// # Arguments
    ///
    /// * `link` - The link.
    pub fn open_link(&self, link: Link)
    {
        self.options.set(link.options);
        self.stage.set(link.stage);
        self.set_ast_mode.set(link.ast_mode);
        self.set_lower_mode.set(link.lower_mode);
        self.load(link.source);
    }

    /// Point at a range from a view, or stop pointing.
    ///
    /// # Arguments
    ///
    /// * `range` - The range, or `None`.
    pub fn point(&self, range: Option<Range>) { self.pointer.set(range) }
}

/// Create the shared state, provide it as context and wire its effects.
///
/// # Returns
///
/// The state.
pub fn provide_state() -> AppState
{
    let linked = url::read_hash().and_then(|hash| permalink::decode(&hash));
    let start = linked.clone().unwrap_or_else(default_link);
    let source = RwSignal::new(start.source);
    let options = RwSignal::new(start.options);
    let input = Signal::derive(move || (source.get(), options.get()));
    let settled: Signal<(String, Options)> = signal_debounced(input, SETTLE_MS);
    let compiled =
        Memo::new(move |_| settled.with(|(text, options)| compile(text, options)));
    let look = provide_look();
    let (ast_mode, set_ast_mode, _) =
        use_local_storage::<AstMode, FromToStringCodec>(AST_MODE_STORE);
    let (lower_mode, set_lower_mode, _) =
        use_local_storage::<LowerMode, FromToStringCodec>(LOWER_MODE_STORE);
    let (editor_share, set_editor_share, _) =
        use_local_storage_with_options::<f64, FromToStringCodec>(
            EDITOR_SHARE_STORE,
            UseStorageOptions::default().initial_value(DEFAULT_EDITOR_SHARE),
        );
    let (stage_share, set_stage_share, _) =
        use_local_storage_with_options::<f64, FromToStringCodec>(
            STAGE_SHARE_STORE,
            UseStorageOptions::default().initial_value(DEFAULT_STAGE_SHARE),
        );
    let (dock_share, set_dock_share, _) =
        use_local_storage_with_options::<f64, FromToStringCodec>(
            DOCK_SHARE_STORE,
            UseStorageOptions::default().initial_value(DEFAULT_DOCK_SHARE),
        );
    let (editor_open, set_editor_open, _) =
        use_local_storage_with_options::<bool, FromToStringCodec>(
            EDITOR_STORE,
            UseStorageOptions::default().initial_value(true),
        );
    let (heap_cap, set_heap_cap, _) =
        use_local_storage::<HeapCap, FromToStringCodec>(HEAP_CAP_STORE);
    let (stage_open, set_stage_open, _) =
        use_local_storage_with_options::<bool, FromToStringCodec>(
            STAGE_STORE,
            UseStorageOptions::default().initial_value(true),
        );
    let (dock_open, set_dock_open, _) =
        use_local_storage_with_options::<bool, FromToStringCodec>(
            DOCK_STORE,
            UseStorageOptions::default().initial_value(true),
        );
    let (problems_open, set_problems_open, _) =
        use_local_storage_with_options::<bool, FromToStringCodec>(
            PROBLEMS_STORE,
            UseStorageOptions::default().initial_value(true),
        );
    let (debug_mode, set_debug_mode, _) =
        use_local_storage::<bool, FromToStringCodec>(DEBUG_STORE);
    let (auto_run, set_auto_run, _) =
        use_local_storage::<bool, FromToStringCodec>(AUTO_RUN_STORE);

    let state = AppState::builder()
        .source(source)
        .options(options)
        .compiled(compiled)
        .pointer(RwSignal::new(None))
        .caret_token(RwSignal::new(None))
        .mouse_token(RwSignal::new(None))
        .stage(RwSignal::new(start.stage))
        .debug(RwSignal::new(Session::default()))
        .editor(StoredValue::new_local(None))
        .mounted(RwSignal::new(false))
        .look(look)
        .ast_mode(ast_mode)
        .set_ast_mode(set_ast_mode)
        .lower_mode(lower_mode)
        .set_lower_mode(set_lower_mode)
        .editor_share(editor_share)
        .set_editor_share(set_editor_share)
        .stage_share(stage_share)
        .set_stage_share(set_stage_share)
        .dock_share(dock_share)
        .set_dock_share(set_dock_share)
        .editor_open(editor_open)
        .set_editor_open(set_editor_open)
        .stage_open(stage_open)
        .set_stage_open(set_stage_open)
        .dock_open(dock_open)
        .set_dock_open(set_dock_open)
        .problems_open(problems_open)
        .set_problems_open(set_problems_open)
        .debug_mode(debug_mode)
        .set_debug_mode(set_debug_mode)
        .auto_step(RwSignal::new(false))
        .heap_cap(heap_cap)
        .set_heap_cap(set_heap_cap)
        .going(RwSignal::new(false))
        .speed(RwSignal::new(Speed::default()))
        .panel_tab(RwSignal::new(PanelTab::default()))
        .heap_focus(RwSignal::new(None))
        .break_lines(RwSignal::new(BTreeSet::new()))
        .op_breaks(RwSignal::new(BTreeSet::new()))
        .auto_run(auto_run)
        .set_auto_run(set_auto_run)
        .tour(RwSignal::new(false))
        .build();
    provide_context(state);
    wire_debug(state);
    wire_editor(state);
    wire_font(state);
    // A link opens its own views over the remembered ones.
    do_if!(
        let Some(link) = &linked,
        state.set_ast_mode.set(link.ast_mode),
        state.set_lower_mode.set(link.lower_mode)
    );
    wire_link(state, settled);
    state
}

/// Start a new debugger session each time the generated ops change, and keep
/// the stops of the break points on source lines in step with the lines and
/// the ops. When
/// debug mode turns on, from the toolbar or the tour, open the last stage
/// that passed and go back to the first op. When it turns off, the run view
/// runs on to the end.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_debug(state: AppState)
{
    let ops = Memo::new(move |_| state.compiled.with(|c| c.ops.clone()));
    Effect::new(move |_| {
        let mut session = ops.with(|ops| Session::new(ops.as_ref()));
        session.set_max_heap(untrack(|| state.heap_cap.get().octets()));
        session.set_line_stops(untrack(|| state.line_stops()));
        session.set_op_stops(untrack(|| state.op_stops()));
        state.debug.set(session);
    });
    // The new session above takes the stops too, so the order of these
    // effects does not matter.
    Effect::new(move |_| {
        let stops = state.line_stops();
        state.debug.update(|s| s.set_line_stops(stops));
    });
    Effect::new(move |_| {
        let cap = state.heap_cap.get().octets();
        state.debug.update(|s| s.set_max_heap(cap));
    });
    Effect::new(move |_| {
        let stops = state.op_stops();
        state.debug.update(|s| s.set_op_stops(stops));
    });
    Effect::new(move |was: Option<bool>| {
        let on = state.debug_mode.get();
        ret_if!(was.is_none_or(|w| w == on) || !on, on);
        state.stage.set(state.compiled.with_untracked(Compiled::last_passed));
        state.auto_step.set(false);
        state.debug.update(Session::reset);
        on
    });
}

/// The shared state of the page.
///
/// # Returns
///
/// The state from context.
pub fn use_state() -> AppState { expect_context::<AppState>() }

/// The link the page opens with when the URL hash holds none: the default
/// sample with the default options, stage and AST view.
///
/// # Returns
///
/// The link.
fn default_link() -> Link
{
    let source = default_sample().text;
    Link::builder().source(source).options(Options::default()).build()
}

/// Keep the editor colors, markers, highlight, theme and break points in
/// step with the state; the gutter for break points shows in debug mode.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_editor(state: AppState)
{
    Effect::new(move |_| {
        ret_if!(!state.mounted.get());
        state.compiled.with(|c| state.with_editor(|editor| editor.show(c)));
    });
    Effect::new(move |_| {
        let range = state.pointer.get();
        ret_if!(!state.mounted.get());
        let span = state
            .compiled
            .with_untracked(|c| range.map(|r| c.table.range_to_utf16(&r)));
        state.with_editor(|editor| editor.set_pointer(span));
    });
    Effect::new(move |_| {
        let dark = state.is_dark();
        ret_if!(!state.mounted.get());
        state.with_editor(|editor| editor.set_dark(dark));
    });
    Effect::new(move |_| {
        let on = state.debug_mode.get();
        let live = |c: &Compiled, l: u32| (l, has_ops(&c.asm, l as usize));
        let marks: Vec<(u32, bool)> = state.break_lines.with(|lines| {
            state.compiled.with(|c| lines.iter().map(|l| live(c, *l)).collect())
        });
        // A line with a break point of its own shows that dot alone.
        let lined = |l: &u32| marks.iter().any(|(m, _)| m == l);
        let ops: Vec<u32> =
            state.op_lines().into_iter().filter(|l| !lined(l)).collect();
        ret_if!(!state.mounted.get());
        state.with_editor(|editor| {
            editor.show_gutter(on);
            editor.show_breaks(&marks);
            editor.show_op_breaks(&ops);
        });
    });
}

/// Keep the editor font in step with the chosen code fonts and size; the
/// shared look keeps the CSS variables.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_font(state: AppState)
{
    Effect::new(move |_| {
        let (pair, size) = (state.look.code_pair(), state.look.font_size.get());
        let stack = pair.code_stack();
        ret_if!(!state.mounted.get());
        leptos::task::spawn_local(async move {
            load_pair(pair, size).await;
            state.with_editor(|editor| editor.set_font(&stack, size));
        });
    });
}

/// Keep the URL hash in step with the settled text and options. It holds
/// still while the tour runs, so a reload keeps the text from before.
///
/// # Arguments
///
/// * `settled` - The text and options after the debounce.
fn wire_link(state: AppState, settled: Signal<(String, Options)>)
{
    Effect::new(move |_| {
        ret_if!(state.tour.get());
        let (stage, ast_mode) = (state.stage.get(), state.ast_mode.get());
        let lower_mode = state.lower_mode.get();
        settled.with(|(text, options)| {
            let link = Link::builder()
                .source(text.clone())
                .options(*options)
                .stage(stage)
                .ast_mode(ast_mode)
                .lower_mode(lower_mode)
                .build();
            url::write_hash(&permalink::encode(&link));
        })
    });
}
