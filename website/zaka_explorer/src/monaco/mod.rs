use bon::Builder;
use js_sys::Array;
use js_sys::Function;
use js_sys::Promise;
use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlElement;
use zakac_i18n::t;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

pub mod decor;

use zaka_web_ui_common::look::line_height;

use crate::compile::Compiled;
use crate::monaco::decor::DiagMark;
use crate::monaco::decor::TokenMark;
use crate::monaco::decor::decorations;
use crate::monaco::decor::markers;

/// The AMD module that defines `monaco`.
const MAIN: &str = "vs/editor/editor.main";
/// Folder of the `vs/...` modules beside the page, named after the version
/// `Trunk.toml` pins, so an upgrade changes each address.
pub const VS_DIR: &str = "monaco/0.57.0/vs";
/// Theme name of the light mode.
const LIGHT: &str = "vs";
/// Theme name of the dark mode.
const DARK: &str = "vs-dark";
/// Owner of the diagnostic markers.
const OWNER: &str = "zakac";
/// Fonts of the editor: the code font from `crate::font`, then fallbacks for
/// Burmese and ASCII while it loads.
pub const FONTS: &str = "\"PDA10 Burmese Mono\", \"Noto Sans Myanmar\", \"Myanmar \
                         MN\", \"Padauk\", ui-monospace, monospace";
/// Class of the text a view points at.
const POINTER_CLASS: &str = "mark-hover";
/// Class of the line a view points at.
const POINTER_LINE_CLASS: &str = "mark-hover-back";
/// Mouse target kind of a spot on the text.
const CONTENT_TEXT: u32 = 6;
/// Mouse target kind of the gutter column left of the line numbers, where
/// break points go.
const GUTTER_MARK: u32 = 2;
/// Class of the dot of a break point on a line with ops.
const BREAK_CLASS: &str = "break-dot";
/// Class of the dot of a break point on a line with no ops.
const BREAK_IDLE_CLASS: &str = "break-dot-idle";
/// Class of the dim dot under the mouse in the gutter column, where a click
/// would set a break point.
const BREAK_GHOST_CLASS: &str = "break-dot-ghost";
/// Class of the mark of a line with a break point on one of its ops.
const BREAK_OP_CLASS: &str = "break-dot-op";
/// Decoration stickiness that never grows when text is written at its edges,
/// so a break point remains on its line.
const NEVER_GROWS: u32 = 1;

#[wasm_bindgen]
extern "C" {
    /// `require.config` of the AMD loader in `vs/loader.js`.
    #[wasm_bindgen(js_namespace = require, js_name = config)]
    fn amd_config(cfg: &JsValue);
    /// `require` of the AMD loader in `vs/loader.js`.
    #[wasm_bindgen(js_name = require)]
    fn amd_load(names: &Array, done: &Function, fail: &Function);

    /// A Monaco editor.
    #[derive(Debug, Clone)]
    type CodeEditor;
    #[wasm_bindgen(catch, js_namespace = ["monaco", "editor"], js_name = create)]
    fn new_editor(host: &HtmlElement, opts: &JsValue) -> Result<CodeEditor, JsValue>;
    #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = setTheme)]
    fn set_theme(name: &str);
    #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = remeasureFonts)]
    fn remeasure_fonts();
    #[wasm_bindgen(js_namespace = ["monaco", "editor"], js_name = setModelMarkers)]
    fn set_model_markers(model: &TextModel, owner: &str, markers: &JsValue);
    #[wasm_bindgen(method, js_name = getModel)]
    fn model(this: &CodeEditor) -> TextModel;
    #[wasm_bindgen(method, js_name = updateOptions)]
    fn update_options(this: &CodeEditor, opts: &JsValue);
    #[wasm_bindgen(method, js_name = createDecorationsCollection)]
    fn new_decor(this: &CodeEditor) -> Decor;
    #[wasm_bindgen(method, js_name = setSelection)]
    fn set_selection(this: &CodeEditor, range: &JsValue);
    #[wasm_bindgen(method, js_name = revealRangeInCenterIfOutsideViewport)]
    fn reveal(this: &CodeEditor, range: &JsValue);
    #[wasm_bindgen(method)]
    fn focus(this: &CodeEditor);
    #[wasm_bindgen(method, js_name = onDidChangeModelContent)]
    fn on_change(this: &CodeEditor, cb: &Closure<dyn FnMut()>);
    #[wasm_bindgen(method, js_name = onDidChangeCursorPosition)]
    fn on_caret_move(this: &CodeEditor, cb: &Closure<dyn FnMut(CaretMove)>);
    #[wasm_bindgen(method, js_name = onMouseMove)]
    fn on_mouse_move(this: &CodeEditor, cb: &Closure<dyn FnMut(MouseEvent)>);
    #[wasm_bindgen(method, js_name = onMouseLeave)]
    fn on_mouse_leave(this: &CodeEditor, cb: &Closure<dyn FnMut()>);
    #[wasm_bindgen(method, js_name = onMouseDown)]
    fn on_mouse_down(this: &CodeEditor, cb: &Closure<dyn FnMut(MouseEvent)>);

    /// The text of an editor.
    #[derive(Debug, Clone)]
    type TextModel;
    #[wasm_bindgen(method, js_name = getValue)]
    fn value(this: &TextModel) -> String;
    #[wasm_bindgen(method, js_name = setValue)]
    fn set_value(this: &TextModel, text: &str);
    #[wasm_bindgen(method, js_name = getPositionAt)]
    fn pos_at(this: &TextModel, off16: u32) -> Pos;
    #[wasm_bindgen(method, js_name = getOffsetAt)]
    fn offset_at(this: &TextModel, pos: &Pos) -> u32;

    /// A set of decorations that is replaced as a whole.
    #[derive(Debug, Clone)]
    type Decor;
    #[wasm_bindgen(method)]
    fn set(this: &Decor, marks: &JsValue);
    #[wasm_bindgen(method)]
    fn clear(this: &Decor);
    #[wasm_bindgen(method, js_name = getRanges)]
    fn ranges(this: &Decor) -> Array;

    /// A range of lines and columns, as Monaco returns it.
    type LineRange;
    #[wasm_bindgen(method, getter = startLineNumber)]
    fn start_line(this: &LineRange) -> u32;

    /// A line and column, both from 1.
    type Pos;
    #[wasm_bindgen(method, getter = lineNumber)]
    fn line(this: &Pos) -> u32;
    #[wasm_bindgen(method, getter)]
    fn column(this: &Pos) -> u32;

    /// The event of a caret move.
    type CaretMove;
    #[wasm_bindgen(method, getter)]
    fn position(this: &CaretMove) -> Pos;

    /// The event of a mouse move over the editor.
    type MouseEvent;
    #[wasm_bindgen(method, getter)]
    fn target(this: &MouseEvent) -> MouseTarget;

    /// What the mouse is over.
    type MouseTarget;
    #[wasm_bindgen(method, getter = "type")]
    fn kind(this: &MouseTarget) -> u32;
    #[wasm_bindgen(method, getter = position)]
    fn spot(this: &MouseTarget) -> Option<Pos>;
}

/// The AMD loader settings.
#[derive(Debug, Serialize, Builder)]
struct AmdConfig
{
    /// Where the modules live.
    paths: AmdPaths,
}

/// Where the AMD modules live.
#[derive(Debug, Serialize, Builder)]
struct AmdPaths
{
    /// The folder of the `vs/...` modules.
    #[builder(default = VS_DIR)]
    vs: &'static str,
}

/// An option that is on or off.
#[derive(Debug, Serialize, Builder)]
struct Toggle
{
    /// On or off.
    enabled: bool,
}

/// How the scroll bars of the editor treat the mouse wheel.
#[derive(Debug, Serialize, Builder)]
struct ScrollOpts
{
    /// Keep each turn of the wheel, even when the text cannot move further.
    /// Off, a turn the editor has no use for scrolls the page around it, as
    /// the stacked panes of narrow screens need.
    #[serde(rename = "alwaysConsumeMouseWheel")]
    hold_wheel: bool,
}

/// Which Unicode text Monaco calls out.
#[derive(Debug, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct UnicodeHighlight
{
    /// Letters past ASCII.
    #[serde(rename = "nonBasicASCII")]
    non_basic_ascii: bool,
    /// Letters that look like others.
    ambiguous_characters: bool,
    /// Letters that take no room.
    invisible_characters: bool,
}

/// Room above the first line and below the last, in px.
#[derive(Debug, Serialize, Builder)]
struct Inset
{
    /// Room above the first line.
    top: u32,
    /// Room below the last line.
    bottom: u32,
}

/// The settings of a new editor.
#[derive(Debug, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct EditorOpts<'a>
{
    /// The starting text.
    value: &'a str,
    /// The theme name.
    theme: &'static str,
    #[builder(default = "plaintext")]
    language: &'static str,
    #[serde(rename = "automaticLayout")]
    #[builder(default = true)]
    auto_fit: bool,
    #[builder(default = Toggle::builder().enabled(false).build())]
    minimap: Toggle,
    #[builder(default = "off")]
    word_wrap: &'static str,
    #[serde(rename = "fontFamily")]
    #[builder(default = FONTS)]
    font_stack: &'a str,
    #[builder(default = 15)]
    font_size: u32,
    #[builder(default = 30)]
    line_height: u32,
    #[builder(default = false)]
    font_ligatures: bool,
    #[builder(default = 4)]
    tab_size: u32,
    #[serde(rename = "scrollBeyondLastLine")]
    #[builder(default = false)]
    scroll_past_end: bool,
    #[builder(default = "line")]
    render_line_highlight: &'static str,
    #[builder(default = Inset::builder().top(12).bottom(12).build())]
    padding: Inset,
    #[builder(default = ScrollOpts::builder().hold_wheel(false).build())]
    scrollbar: ScrollOpts,
    #[builder(default = UnicodeHighlight::builder()
        .non_basic_ascii(false)
        .ambiguous_characters(false)
        .invisible_characters(false)
        .build())]
    unicode_highlight: UnicodeHighlight,
    /// Place hovers and suggestions in fixed position, so the pane around
    /// the editor, which hides its overflow, does not cut them off.
    #[builder(default = true)]
    fixed_overflow_widgets: bool,
    /// The gutter column for break points; debug mode turns it on.
    #[serde(rename = "glyphMargin")]
    #[builder(default = false)]
    gutter_dots: bool,
}

/// Whether the gutter column for break points shows.
#[derive(Debug, Serialize, Builder)]
struct GutterOpts
{
    /// Show the column.
    #[serde(rename = "glyphMargin")]
    gutter_dots: bool,
}

/// A text Monaco shows on hover.
#[derive(Debug, Serialize, Builder)]
struct Note
{
    /// The text, as Markdown.
    value: String,
}

/// The font settings of an editor.
#[derive(Debug, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct FontOpts<'a>
{
    /// The CSS font stack.
    #[serde(rename = "fontFamily")]
    font_stack: &'a str,
    /// The font size in px.
    font_size: u32,
    /// The line height in px.
    line_height: u32,
}

/// A Monaco range between two lines and columns.
#[derive(Debug, Clone, Copy, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct Span16
{
    start_line_number: u32,
    start_column: u32,
    end_line_number: u32,
    end_column: u32,
}

/// How a decoration looks.
#[derive(Debug, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
struct DecorOpts
{
    /// Class of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_class_name: Option<&'static str>,
    /// Class of the whole line.
    #[serde(skip_serializing_if = "Option::is_none")]
    class_name: Option<&'static str>,
    /// Class of a mark in the gutter column for break points.
    #[serde(rename = "glyphMarginClassName", skip_serializing_if = "Option::is_none")]
    dot_class: Option<&'static str>,
    /// The text on hover over that mark.
    #[serde(
        rename = "glyphMarginHoverMessage",
        skip_serializing_if = "Option::is_none"
    )]
    dot_note: Option<Note>,
    /// How the decoration grows with text written at its edges.
    #[serde(skip_serializing_if = "Option::is_none")]
    stickiness: Option<u32>,
}

/// A decoration.
#[derive(Debug, Serialize, Builder)]
struct Mark
{
    /// Where it is.
    range: Span16,
    /// How it looks.
    options: DecorOpts,
}

/// A diagnostic squiggle.
#[derive(Debug, Serialize, Builder)]
struct Marker<'a>
{
    /// Where it is.
    #[serde(flatten)]
    range: Span16,
    /// The Monaco marker level.
    #[serde(rename = "severity")]
    level: u32,
    /// The diagnostic code.
    code: &'a str,
    /// The message.
    message: &'a str,
    /// Who reports it.
    source: &'static str,
}

/// A Monaco editor on the page.
#[derive(Debug, Clone, Builder)]
#[builder(builder_type(vis = ""), start_fn(vis = ""))]
pub struct Editor
{
    /// The Monaco editor.
    code: CodeEditor,
    /// Its text.
    model: TextModel,
    /// The token colors.
    marks: Decor,
    /// The highlight of the range a view points at.
    pointer: Decor,
    /// The break points on source lines.
    breaks: Decor,
    /// The marks of the lines with a break point on one of their ops.
    op_breaks: Decor,
    /// The dim dot under the mouse in the gutter column.
    ghost: Decor,
}

impl Editor
{
    /// Create an editor once Monaco has loaded.
    ///
    /// # Arguments
    ///
    /// * `host` - The element the editor fills.
    /// * `text` - The starting text.
    /// * `dark` - Use the dark theme.
    /// * `font` - The CSS font stack and size in px.
    ///
    /// # Returns
    ///
    /// The editor, or `None` when Monaco failed to load.
    pub async fn create(
        host: &HtmlElement, text: &str, dark: bool, font: (&str, u32),
    ) -> Option<Editor>
    {
        let made = Editor::make(host, text, dark, font).await;
        made.inspect_err(web_sys::console::error_1).ok()
    }

    /// Load Monaco and create an editor.
    ///
    /// # Arguments
    ///
    /// * `host` - The element the editor fills.
    /// * `text` - The starting text.
    /// * `dark` - Use the dark theme.
    /// * `font` - The CSS font stack and size in px.
    ///
    /// # Returns
    ///
    /// The editor, or the error of the loader or of Monaco.
    async fn make(
        host: &HtmlElement, text: &str, dark: bool, (stack, size): (&str, u32),
    ) -> Result<Editor, JsValue>
    {
        load().await?;
        let opts = EditorOpts::builder()
            .value(text)
            .theme(theme(dark))
            .font_stack(stack)
            .font_size(size)
            .line_height(line_height(size))
            .build();
        let code = new_editor(host, &to_js(&opts))?;
        Ok(Editor::builder()
            .model(code.model())
            .marks(code.new_decor())
            .pointer(code.new_decor())
            .breaks(code.new_decor())
            .op_breaks(code.new_decor())
            .ghost(code.new_decor())
            .code(code)
            .build())
    }

    /// Replace the text when it differs; the same text would move the caret.
    ///
    /// # Arguments
    ///
    /// * `text` - The new text.
    pub fn set_text(&self, text: &str)
    {
        ret_if!(self.model.value() == text);
        self.model.set_value(text);
    }

    /// Color the tokens and mark the diagnostics of a compile run.
    ///
    /// # Arguments
    ///
    /// * `compiled` - The compile output.
    pub fn show(&self, compiled: &Compiled)
    {
        let marks = decorations(compiled);
        let marks: Vec<Mark> = marks.iter().map(|m| self.token_mark(m)).collect();
        self.marks.set(&to_js(&marks));
        let found = markers(compiled);
        let found: Vec<Marker> = found.iter().map(|m| self.diag_marker(m)).collect();
        set_model_markers(&self.model, OWNER, &to_js(&found));
    }

    /// Highlight a range, or clear the highlight.
    ///
    /// # Arguments
    ///
    /// * `span` - UTF-16 `(lo, hi)`, or `None` to clear.
    pub fn set_pointer(&self, span: Option<(u32, u32)>)
    {
        let Some((lo, hi)) = span
        else
        {
            return self.pointer.clear();
        };
        let options = DecorOpts::builder()
            .inline_class_name(POINTER_CLASS)
            .class_name(POINTER_LINE_CLASS)
            .build();
        let mark = Mark::builder().range(self.span16(lo, hi)).options(options).build();
        self.pointer.set(&to_js(&[mark]));
    }

    /// Select a range, bring it in view and focus the editor.
    ///
    /// # Arguments
    ///
    /// * `lo` - UTF-16 start.
    /// * `hi` - UTF-16 end.
    pub fn select(&self, lo: u32, hi: u32)
    {
        let range = to_js(&self.span16(lo, hi));
        self.code.set_selection(&range);
        self.code.reveal(&range);
        self.code.focus();
    }

    /// Switch between the light and the dark theme.
    ///
    /// # Arguments
    ///
    /// * `dark` - Use the dark theme.
    pub fn set_dark(&self, dark: bool) { set_theme(theme(dark)) }

    /// Switch the font, then have Monaco measure it again.
    ///
    /// # Arguments
    ///
    /// * `stack` - The CSS font stack.
    /// * `size` - The font size in px.
    pub fn set_font(&self, stack: &str, size: u32)
    {
        let opts = FontOpts::builder()
            .font_stack(stack)
            .font_size(size)
            .line_height(line_height(size))
            .build();
        self.code.update_options(&to_js(&opts));
        remeasure_fonts();
    }

    /// Call `f` with the text after each edit.
    ///
    /// # Arguments
    ///
    /// * `f` - The callback.
    pub fn on_edit(&self, mut f: impl FnMut(String) + 'static)
    {
        let model = self.model.clone();
        let cb = Closure::<dyn FnMut()>::new(move || f(model.value()));
        self.code.on_change(&cb);
        cb.forget();
    }

    /// Call `f` with the UTF-16 caret offset when the caret moves.
    ///
    /// # Arguments
    ///
    /// * `f` - The callback.
    pub fn on_caret(&self, mut f: impl FnMut(u32) + 'static)
    {
        let model = self.model.clone();
        let cb = Closure::<dyn FnMut(CaretMove)>::new(move |e: CaretMove| {
            f(model.offset_at(&e.position()))
        });
        self.code.on_caret_move(&cb);
        cb.forget();
    }

    /// Call `f` with the UTF-16 offset under the mouse, `None` off the text.
    ///
    /// # Arguments
    ///
    /// * `f` - The callback.
    pub fn on_point(&self, f: impl Fn(Option<u32>) + Clone + 'static)
    {
        let model = self.model.clone();
        let leave = f.clone();
        let on_move = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            f(point_at(&model, &e))
        });
        let on_leave = Closure::<dyn FnMut()>::new(move || leave(None));
        self.code.on_mouse_move(&on_move);
        self.code.on_mouse_leave(&on_leave);
        on_move.forget();
        on_leave.forget();
    }

    /// Draw the break points on source lines in the gutter.
    ///
    /// # Arguments
    ///
    /// * `lines` - Each 1-based line, and whether it has ops to stop at.
    pub fn show_breaks(&self, lines: &[(u32, bool)])
    {
        let marks: Vec<Mark> =
            lines.iter().map(|&(l, live)| break_mark(l, live)).collect();
        self.breaks.set(&to_js(&marks));
    }

    /// Mark the lines with a break point on one of their ops, apart from the
    /// break points on lines, so an edit does not read them back as those.
    ///
    /// # Arguments
    ///
    /// * `lines` - The 1-based lines.
    pub fn show_op_breaks(&self, lines: &[u32])
    {
        let note = t!("explorer.editor.break_op").to_string();
        let mark = |&l: &u32| line_mark(l, BREAK_OP_CLASS, note.clone());
        let marks: Vec<Mark> = lines.iter().map(mark).collect();
        self.op_breaks.set(&to_js(&marks));
    }

    /// The lines of the break points, where Monaco has moved them with the
    /// edits.
    ///
    /// # Returns
    ///
    /// The 1-based lines, as Monaco holds them.
    pub fn break_lines(&self) -> Vec<u32> { break_lines_of(&self.breaks) }

    /// Show or hide the gutter column for break points.
    ///
    /// # Arguments
    ///
    /// * `on` - Show it.
    pub fn show_gutter(&self, on: bool)
    {
        let opts = GutterOpts::builder().gutter_dots(on).build();
        self.code.update_options(&to_js(&opts));
    }

    /// Call `f` with the 1-based line of a click in the gutter column for
    /// break points; the dim dot under the mouse goes, as the click sets or
    /// clears a real one.
    ///
    /// # Arguments
    ///
    /// * `f` - The callback.
    pub fn on_gutter_click(&self, f: impl Fn(u32) + 'static)
    {
        let ghost = self.ghost.clone();
        let cb = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            let target = e.target();
            ret_if!(target.kind() != GUTTER_MARK);
            ghost.clear();
            do_if!(let Some(pos) = target.spot(), f(pos.line()));
        });
        self.code.on_mouse_down(&cb);
        cb.forget();
    }

    /// Show a dim dot on the line under the mouse in the gutter column for
    /// break points, where a click would set one; none on a line that has
    /// one, nor off the column.
    pub fn follow_gutter(&self)
    {
        let ghost = self.ghost.clone();
        let breaks = self.breaks.clone();
        let note = t!("explorer.editor.break_add").to_string();
        let on_move = Closure::<dyn FnMut(MouseEvent)>::new(move |e: MouseEvent| {
            let target = e.target();
            let line = target
                .spot()
                .map(|p| p.line())
                .filter(|_| target.kind() == GUTTER_MARK);
            let taken = |l: &u32| break_lines_of(&breaks).contains(l);
            let Some(line) = line.filter(|l| !taken(l))
            else
            {
                return ghost.clear();
            };
            ghost.set(&to_js(&[line_mark(line, BREAK_GHOST_CLASS, note.clone())]));
        });
        let gone = self.ghost.clone();
        let on_leave = Closure::<dyn FnMut()>::new(move || gone.clear());
        self.code.on_mouse_move(&on_move);
        self.code.on_mouse_leave(&on_leave);
        on_move.forget();
        on_leave.forget();
    }

    /// The Monaco range between two UTF-16 offsets.
    ///
    /// # Arguments
    ///
    /// * `lo` - UTF-16 start.
    /// * `hi` - UTF-16 end.
    ///
    /// # Returns
    ///
    /// The range.
    fn span16(&self, lo: u32, hi: u32) -> Span16
    {
        let (start, end) = (self.model.pos_at(lo), self.model.pos_at(hi));
        Span16::builder()
            .start_line_number(start.line())
            .start_column(start.column())
            .end_line_number(end.line())
            .end_column(end.column())
            .build()
    }

    /// The decoration of a token color.
    ///
    /// # Arguments
    ///
    /// * `mark` - The token color.
    ///
    /// # Returns
    ///
    /// The decoration.
    fn token_mark(&self, mark: &TokenMark) -> Mark
    {
        let options = DecorOpts::builder().inline_class_name(mark.class).build();
        Mark::builder().range(self.span16(mark.lo, mark.hi)).options(options).build()
    }

    /// The Monaco marker of a diagnostic squiggle.
    ///
    /// # Arguments
    ///
    /// * `mark` - The squiggle.
    ///
    /// # Returns
    ///
    /// The marker.
    fn diag_marker<'a>(&self, mark: &'a DiagMark) -> Marker<'a>
    {
        Marker::builder()
            .range(self.span16(mark.lo, mark.hi))
            .level(mark.level)
            .code(&mark.code)
            .message(&mark.message)
            .source(OWNER)
            .build()
    }
}

/// The decoration of a break point on a source line.
///
/// # Arguments
///
/// * `line` - The 1-based line.
/// * `live` - Whether the line has ops to stop at.
///
/// # Returns
///
/// The decoration.
fn break_mark(line: u32, live: bool) -> Mark
{
    let (class, note) = tern!(
        live,
        (BREAK_CLASS, t!("explorer.editor.break")),
        (BREAK_IDLE_CLASS, t!("explorer.editor.break_idle"))
    );
    line_mark(line, class, note.to_string())
}

/// The lines of a set of break point marks.
///
/// # Arguments
///
/// * `breaks` - The marks.
///
/// # Returns
///
/// The 1-based lines, as Monaco holds them.
fn break_lines_of(breaks: &Decor) -> Vec<u32>
{
    let line = |r: JsValue| r.unchecked_into::<LineRange>().start_line();
    breaks.ranges().into_iter().map(line).collect()
}

/// A mark in the gutter column for break points.
///
/// # Arguments
///
/// * `line` - The 1-based line.
/// * `class` - The class of the mark.
/// * `note` - The text on hover.
///
/// # Returns
///
/// The decoration.
fn line_mark(line: u32, class: &'static str, note: String) -> Mark
{
    let options = DecorOpts::builder()
        .dot_class(class)
        .dot_note(Note::builder().value(note).build())
        .stickiness(NEVER_GROWS)
        .build();
    let range = Span16::builder()
        .start_line_number(line)
        .start_column(1)
        .end_line_number(line)
        .end_column(1)
        .build();
    Mark::builder().range(range).options(options).build()
}

/// Load Monaco with the AMD loader.
///
/// # Returns
///
/// The error of the loader when it fails.
async fn load() -> Result<(), JsValue>
{
    add_loader().await?;
    let cfg = AmdConfig::builder().paths(AmdPaths::builder().build()).build();
    amd_config(&to_js(&cfg));
    let names = Array::of1(&JsValue::from_str(MAIN));
    let wait = Promise::new(&mut |done, fail| amd_load(&names, &done, &fail));
    JsFuture::from(wait).await.map(drop)
}

/// Add the AMD loader to the page and wait for it, unless the page holds it
/// from the start, as `index.html` does. A host page with UMD scripts adds it
/// late, once those have run, so its `define` does not catch them.
///
/// # Returns
///
/// The error of the browser when the script fails to load.
async fn add_loader() -> Result<(), JsValue>
{
    let has = js_sys::Reflect::has(&js_sys::global(), &JsValue::from_str("require"))?;
    ret_if!(has, Ok(()));
    let doc = leptos::prelude::document();
    let script = doc.create_element("script")?;
    script.set_attribute("src", &format!("{VS_DIR}/loader.js"))?;
    let wait = Promise::new(&mut |done, fail| {
        script.add_event_listener_with_callback("load", &done).ok();
        script.add_event_listener_with_callback("error", &fail).ok();
    });
    let head = doc.head().ok_or_else(|| JsValue::from_str("the page has no head"))?;
    head.append_child(&script)?;
    JsFuture::from(wait).await.map(drop)
}

/// The UTF-16 offset under the mouse.
///
/// # Arguments
///
/// * `model` - The text.
/// * `e` - The mouse event.
///
/// # Returns
///
/// The offset, or `None` off the text.
fn point_at(model: &TextModel, e: &MouseEvent) -> Option<u32>
{
    let target = e.target();
    ret_if!(target.kind() != CONTENT_TEXT, None);
    target.spot().map(|pos| model.offset_at(&pos))
}

/// The theme name of a mode.
///
/// # Arguments
///
/// * `dark` - The dark mode.
///
/// # Returns
///
/// The name.
fn theme(dark: bool) -> &'static str { tern!(dark, DARK, LIGHT) }

/// A Rust value as a plain JS object.
///
/// # Arguments
///
/// * `value` - The value.
///
/// # Returns
///
/// The object; `undefined` when it does not convert, which is logged.
fn to_js(value: &impl Serialize) -> JsValue
{
    let out = value.serialize(&Serializer::json_compatible());
    out.unwrap_or_else(|err| {
        web_sys::console::error_1(&err.into());
        JsValue::UNDEFINED
    })
}
