use bon::Builder;
use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use_color_mode::ColorMode;
use leptos_use_color_mode::UseColorModeOptions;
use leptos_use_color_mode::UseColorModeReturn;
use leptos_use_color_mode::use_color_mode_with_options;
use leptos_use_css_var::use_css_var;
use leptos_use_signal_debounce_throttle::signal_debounced;
use leptos_use_storage::UseStorageOptions;
use leptos_use_storage::use_local_storage;
use leptos_use_storage::use_local_storage_with_options;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

pub mod lang;
pub mod permalink;
pub mod settings;
pub mod split;
pub mod url;

use crate::compile::Compiled;
use crate::compile::Options;
use crate::compile::Phase;
use crate::compile::compile;
use crate::compile::samples::default_sample;
use crate::compile::spans::Range;
use crate::font::face::Face;
use crate::font::load_face;
use crate::monaco::Editor;
use crate::state::permalink::Link;
use crate::state::settings::AstMode;
use crate::state::settings::CUSTOM_MODES;
use crate::state::settings::LowerMode;
use crate::state::settings::Theme;
use crate::state::settings::font_size_or_default;
use crate::state::split::DEFAULT_EDITOR_SHARE;
use crate::state::split::DEFAULT_STAGE_SHARE;

/// Milliseconds of quiet after an edit before the phases run again.
const SETTLE_MS: f64 = 150.0;

/// Local storage name of the color mode.
const THEME_STORE: &str = "zaka-explorer-theme";

/// Local storage name of the code font.
const FONT_STORE: &str = "zaka-explorer-font";

/// Size in px the interface font is loaded at.
const UI_SIZE: u32 = 14;

/// Local storage name of the interface font.
const UI_FONT_STORE: &str = "zaka-explorer-ui-font";

/// Local storage name of the PDA10-for-Burmese switch.
const PDA_STORE: &str = "zaka-explorer-pda-burmese";

/// Local storage name of the AST view mode.
const AST_MODE_STORE: &str = "zaka-explorer-ast-mode";

/// Local storage name of the lowering view mode.
const LOWER_MODE_STORE: &str = "zaka-explorer-lower-mode";

/// Local storage name of the italic comments switch.
const ITALIC_STORE: &str = "zaka-explorer-italic-comments";

/// Local storage name of the code font size.
const SIZE_STORE: &str = "zaka-explorer-font-size";

/// Local storage name of the editor share of the width.
const EDITOR_SHARE_STORE: &str = "zaka-explorer-editor-share";

/// Local storage name of the stage view share of the right column.
const STAGE_SHARE_STORE: &str = "zaka-explorer-stage-share";

/// Local storage name of the editor pane switch.
const EDITOR_STORE: &str = "zaka-explorer-editor-open";

/// Local storage name of the problems panel switch.
const PROBLEMS_STORE: &str = "zaka-explorer-problems-open";

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
    /// The editor, once Monaco has loaded.
    pub editor: StoredValue<Option<Editor>, LocalStorage>,
    /// Whether the editor has loaded.
    pub mounted: RwSignal<bool>,
    /// The chosen color mode; `Auto` follows the OS.
    pub mode: Signal<ColorMode>,
    /// Choose the color mode.
    pub set_mode: WriteSignal<ColorMode>,
    /// The color mode on the page; never `Auto`.
    pub resolved: Signal<ColorMode>,
    /// The font of interface text.
    pub ui_font: Signal<Face>,
    /// Choose the font of interface text.
    pub set_ui_font: WriteSignal<Face>,
    /// The font of code text.
    pub code_font: Signal<Face>,
    /// Choose the font of code text.
    pub set_code_font: WriteSignal<Face>,
    /// Draw Burmese text in PDA10 Burmese Mono whatever the code font.
    pub pda_burmese: Signal<bool>,
    /// Switch PDA10 for Burmese text on or off.
    pub set_pda_burmese: WriteSignal<bool>,
    /// How the AST view shows the tree.
    pub ast_mode: Signal<AstMode>,
    /// Choose how the AST view shows the tree.
    pub set_ast_mode: WriteSignal<AstMode>,
    /// Which IR the lowering view shows.
    pub lower_mode: Signal<LowerMode>,
    /// Choose which IR the lowering view shows.
    pub set_lower_mode: WriteSignal<LowerMode>,
    /// Draw comments in italics in the editor.
    pub italic_comments: Signal<bool>,
    /// Switch italic comments on or off.
    pub set_italic_comments: WriteSignal<bool>,
    /// The size of code text in px.
    pub font_size: Signal<u32>,
    /// Choose the size of code text.
    pub set_font_size: WriteSignal<u32>,
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
    pub fn is_dark(&self) -> bool { Theme::is_dark(&self.resolved.get()) }

    /// The CSS font stack and size of code text, without tracking.
    ///
    /// # Returns
    ///
    /// The stack and the size in px.
    pub fn font_untracked(&self) -> (String, u32)
    {
        let stack =
            self.code_font.get_untracked().code_stack(self.pda_burmese.get_untracked());
        (stack, self.font_size.get_untracked())
    }

    /// The CSS font stack of code text.
    ///
    /// # Returns
    ///
    /// The stack.
    pub fn code_stack(&self) -> String
    {
        self.code_font.get().code_stack(self.pda_burmese.get())
    }

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

    /// Write the link of the current text to the URL at once, ahead of the
    /// debounce, so a reload right after keeps the latest edits.
    pub fn flush_link(&self)
    {
        let link = Link::builder()
            .source(self.source.get_untracked())
            .options(self.options.get_untracked())
            .stage(self.stage.get_untracked())
            .ast_mode(self.ast_mode.get_untracked())
            .lower_mode(self.lower_mode.get_untracked())
            .build();
        url::write_hash(&permalink::encode(&link));
    }

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
    let theme = UseColorModeOptions::default()
        .attribute("data-theme")
        .storage_key(THEME_STORE)
        .custom_modes(CUSTOM_MODES.map(String::from).to_vec())
        .emit_auto(true);
    let UseColorModeReturn { mode, set_mode, state: resolved, .. } =
        use_color_mode_with_options(theme);
    let (stored_ui, set_ui_font, _) =
        use_local_storage_with_options::<Face, FromToStringCodec>(
            UI_FONT_STORE,
            UseStorageOptions::default().initial_value(Face::default_ui()),
        );
    let ui_font = Signal::derive(move || Face::ui_or_default(stored_ui.get()));
    let (stored_font, set_code_font, _) =
        use_local_storage::<Face, FromToStringCodec>(FONT_STORE);
    let code_font = Signal::derive(move || Face::code_or_default(stored_font.get()));
    let (pda_burmese, set_pda_burmese, _) =
        use_local_storage::<bool, FromToStringCodec>(PDA_STORE);
    let (ast_mode, set_ast_mode, _) =
        use_local_storage::<AstMode, FromToStringCodec>(AST_MODE_STORE);
    let (lower_mode, set_lower_mode, _) =
        use_local_storage::<LowerMode, FromToStringCodec>(LOWER_MODE_STORE);
    let (italic_comments, set_italic_comments, _) =
        use_local_storage::<bool, FromToStringCodec>(ITALIC_STORE);
    let (stored_size, set_font_size, _) =
        use_local_storage::<u32, FromToStringCodec>(SIZE_STORE);
    let font_size = Signal::derive(move || font_size_or_default(stored_size.get()));
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

    let state = AppState::builder()
        .source(source)
        .options(options)
        .compiled(compiled)
        .pointer(RwSignal::new(None))
        .caret_token(RwSignal::new(None))
        .mouse_token(RwSignal::new(None))
        .stage(RwSignal::new(start.stage))
        .editor(StoredValue::new_local(None))
        .mounted(RwSignal::new(false))
        .mode(mode)
        .set_mode(set_mode)
        .resolved(resolved)
        .ui_font(ui_font)
        .set_ui_font(set_ui_font)
        .code_font(code_font)
        .set_code_font(set_code_font)
        .pda_burmese(pda_burmese)
        .set_pda_burmese(set_pda_burmese)
        .ast_mode(ast_mode)
        .set_ast_mode(set_ast_mode)
        .lower_mode(lower_mode)
        .set_lower_mode(set_lower_mode)
        .italic_comments(italic_comments)
        .set_italic_comments(set_italic_comments)
        .font_size(font_size)
        .set_font_size(set_font_size)
        .editor_share(editor_share)
        .set_editor_share(set_editor_share)
        .stage_share(stage_share)
        .set_stage_share(set_stage_share)
        .editor_open(editor_open)
        .set_editor_open(set_editor_open)
        .problems_open(problems_open)
        .set_problems_open(set_problems_open)
        .build();
    provide_context(state);
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

/// Keep the CSS variables of interface and code text and the editor font in
/// step with the chosen fonts and size.
///
/// # Arguments
///
/// * `state` - The state.
fn wire_font(state: AppState)
{
    let (_, set_stack) = use_css_var("--mono-font");
    let (_, set_size) = use_css_var("--code-size");
    let (_, set_ui_stack) = use_css_var("--ui-font");
    let (_, set_comment_style) = use_css_var("--comment-style");
    Effect::new(move |_| {
        set_comment_style
            .set(tern!(state.italic_comments.get(), "italic", "normal").into());
    });
    Effect::new(move |_| {
        let face = state.ui_font.get();
        set_ui_stack.set(face.ui_stack());
        leptos::task::spawn_local(load_face(face, UI_SIZE));
    });
    Effect::new(move |_| {
        let (face, size) = (state.code_font.get(), state.font_size.get());
        set_stack.set(state.code_stack());
        set_size.set(format!("{size}px"));
        leptos::task::spawn_local(load_face(face, size));
    });
    Effect::new(move |_| {
        let (font, size) = (state.code_font.get(), state.font_size.get());
        let stack = state.code_stack();
        ret_if!(!state.mounted.get());
        leptos::task::spawn_local(async move {
            load_face(font, size).await;
            state.with_editor(|editor| editor.set_font(&stack, size));
        });
    });
}

/// Keep the URL hash in step with the settled text and options.
///
/// # Arguments
///
/// * `settled` - The text and options after the debounce.
fn wire_link(state: AppState, settled: Signal<(String, Options)>)
{
    Effect::new(move |_| {
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
