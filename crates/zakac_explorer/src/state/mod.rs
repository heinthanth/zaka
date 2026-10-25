use bon::Builder;
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use_signal_debounce_throttle::signal_debounced;
use leptos_use_storage::UseStorageOptions;
use leptos_use_storage::use_local_storage;
use leptos_use_storage::use_local_storage_with_options;
use zaka_web_ui_common::font::load_face;
pub use zaka_web_ui_common::lang;
use zaka_web_ui_common::look::Look;
use zaka_web_ui_common::look::provide_look;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
pub mod permalink;
pub mod settings;
pub mod split;
pub mod tour;
pub mod url;

use crate::compile::Compiled;
use crate::compile::Options;
use crate::compile::Phase;
use crate::compile::compile;
use crate::compile::samples::default_sample;
use crate::compile::spans::Range;
use crate::debug::Session;
use crate::monaco::Editor;
use crate::state::permalink::Link;
use crate::state::settings::AstMode;
use crate::state::settings::LowerMode;
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

/// Local storage name of the editor pane switch.
const EDITOR_STORE: &str = "zaka-explorer-editor-open";

/// Local storage name of the problems panel switch.
const PROBLEMS_STORE: &str = "zaka-explorer-problems-open";

/// Local storage name of the debug mode switch of the run view.
const DEBUG_STORE: &str = "zaka-explorer-run-debug";

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
    /// Whether the editor pane shows.
    pub editor_open: Signal<bool>,
    /// Show or hide the editor pane.
    pub set_editor_open: WriteSignal<bool>,
    /// Whether the problems panel is open.
    pub problems_open: Signal<bool>,
    /// Open or close the problems panel.
    pub set_problems_open: WriteSignal<bool>,
    /// Whether the run view is in debug mode.
    pub run_debug: Signal<bool>,
    /// Turn debug mode of the run view on or off.
    pub set_run_debug: WriteSignal<bool>,
    /// Whether the debugger steps on a timer.
    pub auto_step: RwSignal<bool>,
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
        self.with_editor(|editor| editor.set_text(&text));
        self.source.set(text);
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
    let (editor_open, set_editor_open, _) =
        use_local_storage_with_options::<bool, FromToStringCodec>(
            EDITOR_STORE,
            UseStorageOptions::default().initial_value(true),
        );
    let (problems_open, set_problems_open, _) =
        use_local_storage_with_options::<bool, FromToStringCodec>(
            PROBLEMS_STORE,
            UseStorageOptions::default().initial_value(true),
        );
    let (run_debug, set_run_debug, _) =
        use_local_storage::<bool, FromToStringCodec>(DEBUG_STORE);

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
        .editor_open(editor_open)
        .set_editor_open(set_editor_open)
        .problems_open(problems_open)
        .set_problems_open(set_problems_open)
        .run_debug(run_debug)
        .set_run_debug(set_run_debug)
        .auto_step(RwSignal::new(false))
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

/// Start a new debugger session each time the generated ops change.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_debug(state: AppState)
{
    let ops = Memo::new(move |_| state.compiled.with(|c| c.ops.clone()));
    Effect::new(move |_| ops.with(|ops| state.debug.set(Session::new(ops.as_ref()))));
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

/// Keep the editor colors, markers, highlight and theme in step with the
/// state.
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
}

/// Keep the editor font in step with the chosen code font and size; the
/// shared look keeps the CSS variables.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_font(state: AppState)
{
    Effect::new(move |_| {
        let (font, size) = (state.look.code_font.get(), state.look.font_size.get());
        let stack = state.code_stack();
        ret_if!(!state.mounted.get());
        leptos::task::spawn_local(async move {
            load_face(font, size).await;
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
