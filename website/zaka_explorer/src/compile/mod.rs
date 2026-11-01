use std::borrow::Cow;

use bon::Builder;
use strum::EnumIter;
use strum::EnumString;
use strum::IntoEnumIterator;
use strum::IntoStaticStr;
use zaka_vm::trap::Trap;
pub use zaka_web_ui_common::paint::token_class;
use zakac_ast::s_expr::ToSexpr;
use zakac_borrow_checker::borrow_check;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::MemCheck;
use zakac_borrow_checker::options::OverflowMode;
use zakac_burmese_segmenter::segmenter::PreLexicalToken;
use zakac_burmese_segmenter::segmenter::SegmenterOptions;
use zakac_burmese_segmenter::segmenter::segment_at;
use zakac_codegen::bc;
use zakac_codegen::emit::emit;
use zakac_codegen::listing::format_text;
use zakac_codegen::options::EmitOptions;
use zakac_codegen::options::ResultMode;
use zakac_codegen::options::ScratchMode;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Label;
use zakac_diagnostics::Level;
use zakac_diagnostics::has_errors;
use zakac_i18n::t;
use zakac_ir_builder::fold::fold;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::LowerOptions;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::lower::lower;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_lexer::token::LexOutput;
use zakac_lexer::token::Particle;
use zakac_lexer::token::StrPart;
use zakac_lexer::token::Token;
use zakac_lexer::token::TokenKind;
use zakac_naturalness::NaturalnessOptions;
use zakac_naturalness::check;
use zakac_parser::load::refuse_imports;
use zakac_parser::parser::parse;
use zakac_reporter::render::Piece;
use zakac_reporter::render::diagnostic_pieces;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_semcheck::semcheck::SemcheckOutput;
use zakac_semcheck::semcheck::semcheck;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;
use zakac_span::span::Span;

pub mod asm;
pub mod graph;
pub mod ir_tree;
pub mod s_expr;
pub mod samples;
pub mod spans;
pub mod timing;
pub mod tree;

use crate::compile::asm::AsmFunc;
use crate::compile::asm::SlotRow;
use crate::compile::asm::asm_rows;
use crate::compile::asm::slot_rows;
use crate::compile::ir_tree::build_ir_tree;
use crate::compile::spans::OffsetTable;
use crate::compile::spans::Range;
use crate::compile::spans::find_in;
use crate::compile::spans::to_original;
use crate::compile::timing::PhaseTimes;
use crate::compile::timing::timed;
use crate::compile::tree::TreeNode;
use crate::compile::tree::build_tree;

/// File name shown in diagnostics.
pub const FILE_NAME: &str = "explorer.zaka";

/// Platform bit counts the check accepts.
pub const PLATFORM_BITS: [u16; 5] = [8, 16, 32, 64, 128];

/// Options of a compile run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct Options
{
    /// Bit count of the platform-sized integer kinds.
    #[builder(default = 64)]
    pub platform_bits: u16,
    /// Which foldings rewrite the IR.
    #[builder(default)]
    pub fold: FoldOptions,
    /// What a pointer mistake the ownership check proves does.
    #[builder(default)]
    pub check: MemCheck,
    /// Also warn about a pointer that can be blank.
    #[builder(default)]
    pub strict: bool,
    /// Reject each kind written with a raw address.
    #[builder(default)]
    pub no_raw: bool,
    /// What integer arithmetic does with a result outside its kind.
    #[builder(default)]
    pub overflow: OverflowMode,
    /// How a list or struct result reaches the caller.
    #[builder(default)]
    pub results: ResultMode,
    /// Where the scratch values of a function live.
    #[builder(default)]
    pub scratch: ScratchMode,
}

/// The code generation choices of the compile options.
///
/// # Arguments
///
/// * `options` - The compile options.
///
/// # Returns
///
/// The choices.
fn emit_options(options: &Options) -> EmitOptions
{
    EmitOptions::builder().results(options.results).scratch(options.scratch).build()
}

impl Default for Options
{
    /// A 64-bit platform.
    ///
    /// # Returns
    ///
    /// The options.
    fn default() -> Self { Options::builder().build() }
}

/// A compiler phase. It parses from its static name in upper or lower case,
/// as links write it.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    EnumIter,
    IntoStaticStr,
    EnumString
)]
#[strum(ascii_case_insensitive)]
pub enum Phase
{
    /// Pre-lexical segmentation.
    Segment,
    /// The lexer.
    #[default]
    Lex,
    /// The parser.
    Parse,
    /// The kind check.
    Check,
    /// Lowering to the IR and constant folding.
    Lower,
    /// The ownership and loan check of the folded IR.
    BorrowCheck,
    /// Code generation for the stack-based virtual machine.
    Codegen,
    /// A run of the ops in the VM, one step at a time.
    Run,
}

impl Phase
{
    /// Stage label in the pipeline.
    ///
    /// # Returns
    ///
    /// The label.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            Phase::Segment => t!("explorer.phase.segment"),
            Phase::Lex => t!("explorer.phase.lex"),
            Phase::Parse => t!("explorer.phase.parse"),
            Phase::Check => t!("explorer.phase.check"),
            Phase::Lower => t!("explorer.phase.lower"),
            Phase::BorrowCheck => t!("explorer.phase.borrow_check"),
            Phase::Codegen => t!("explorer.phase.codegen"),
            Phase::Run => t!("explorer.phase.run"),
        }
    }

    /// The part of the compiler that runs the phase, as the pipeline names
    /// its stage.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn component(self) -> Cow<'static, str>
    {
        match self
        {
            Phase::Segment => t!("explorer.phase.segment_component"),
            Phase::Lex => t!("explorer.phase.lex_component"),
            Phase::Parse => t!("explorer.phase.parse_component"),
            Phase::Check => t!("explorer.phase.check_component"),
            Phase::Lower => t!("explorer.phase.lower_component"),
            Phase::BorrowCheck => t!("explorer.phase.borrow_check_component"),
            Phase::Codegen => t!("explorer.phase.codegen_component"),
            Phase::Run => t!("explorer.phase.run_component"),
        }
    }

    /// The output the stage shows, under its label.
    ///
    /// # Returns
    ///
    /// The note.
    pub fn note(self) -> Cow<'static, str>
    {
        match self
        {
            Phase::Segment => t!("explorer.phase.segment_note"),
            Phase::Lex => t!("explorer.phase.lex_note"),
            Phase::Parse => t!("explorer.phase.parse_note"),
            Phase::Check => t!("explorer.phase.check_note"),
            Phase::Lower => t!("explorer.phase.lower_note"),
            Phase::BorrowCheck => t!("explorer.phase.borrow_check_note"),
            Phase::Codegen => t!("explorer.phase.codegen_note"),
            Phase::Run => t!("explorer.phase.run_note"),
        }
    }
}

/// How a phase went; its static name is a CSS class suffix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Status
{
    /// No diagnostics.
    Clean,
    /// Warnings, but no errors.
    Warned,
    /// At least one error.
    Failed,
    /// The phase did not run.
    Skipped,
}

impl Status
{
    /// The word for it in the stage tooltip.
    ///
    /// # Returns
    ///
    /// The word.
    pub fn label(self) -> Cow<'static, str>
    {
        match self
        {
            Status::Clean => t!("explorer.status.clean"),
            Status::Warned => t!("explorer.status.warned"),
            Status::Failed => t!("explorer.status.failed"),
            Status::Skipped => t!("explorer.status.skipped"),
        }
    }
}

/// A pre-lexical segment.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct SegmentRow
{
    /// The segment kind.
    pub kind: &'static str,
    /// The text.
    pub lexeme: String,
    /// The source region.
    pub range: Range,
    /// 1-based line.
    pub line: usize,
    /// 1-based column, in chars.
    pub col: usize,
}

/// A token of the lexer.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct TokenRow
{
    /// The token form, such as `kw` or `ident`.
    pub kind: &'static str,
    /// The reserved word, operator, number shape or decoded value.
    #[builder(default)]
    pub detail: String,
    /// The absorbed sentence-final particle, blank for none.
    #[builder(default)]
    pub particle: String,
    /// No whitespace before the token.
    pub joined: bool,
    /// The text the lexer saw.
    pub lexeme: String,
    /// The source region.
    pub range: Range,
    /// 1-based line.
    pub line: usize,
    /// 1-based column, in chars.
    pub col: usize,
    /// CSS class that colors the token in the editor.
    pub class: &'static str,
}

/// A comment of the lexer.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct CommentRow
{
    /// The marker form.
    pub kind: &'static str,
    /// The text, marker included.
    pub text: String,
    /// The source region.
    pub range: Range,
    /// 1-based line.
    pub line: usize,
}

/// What a piece of a string literal is; its static name is a CSS class
/// suffix and the tooltip of the piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum PartKind
{
    /// Plain text.
    Text,
    /// An escape such as `\n`.
    Escape,
    /// A `{...}` template slot: an expression whose value takes its place.
    TemplateSlot,
}

/// A piece of a decoded string literal, as the strings table shows it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct StrPiece
{
    /// What the piece is.
    pub kind: PartKind,
    /// The piece as it reads: the text, the escape as written, or the template
    /// slot in braces.
    #[builder(into)]
    pub text: String,
}

/// A decoded string literal.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct StrRow
{
    /// Index in the string table.
    pub idx: usize,
    /// The decoded value in pieces; the view draws escapes and template slots
    /// as chips, so a newline reads `\n`.
    pub value: Vec<StrPiece>,
    /// The literal as written in the source, quotes included.
    pub source: String,
    /// The source region of the literal.
    pub range: Range,
}

/// A diagnostic of a phase.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct DiagRow
{
    /// The phase that reported it.
    pub phase: Phase,
    /// How serious it is.
    pub level: Level,
    /// The code, such as `ZK0301`.
    pub code: String,
    /// The message.
    pub message: String,
    /// The hint, when there is one.
    pub help: Option<String>,
    /// The source region; `None` for a placeholder span.
    pub range: Option<Range>,
    /// The other regions it points at, each with its note, such as the later
    /// use of a freed block; placeholder spans are left out.
    #[builder(default)]
    pub labels: Vec<(Range, String)>,
    /// The CLI rendering, in painted pieces.
    pub pieces: Vec<Piece>,
}

/// What the views show for one source text.
#[derive(Debug, Clone, PartialEq, Default, Builder)]
pub struct Compiled
{
    /// The text that was compiled.
    pub source: String,
    /// UTF-8 to UTF-16 offsets of `source`.
    pub table: OffsetTable,
    /// Segments, in source order.
    pub segments: Vec<SegmentRow>,
    /// Tokens without the end marker, in source order.
    pub tokens: Vec<TokenRow>,
    /// Comments, in source order.
    pub comments: Vec<CommentRow>,
    /// Decoded string literals.
    pub strings: Vec<StrRow>,
    /// The tree view of the program.
    pub tree: TreeNode,
    /// The S-expression form of the program.
    pub sexpr: String,
    /// Diagnostics of all phases.
    pub diagnostics: Vec<DiagRow>,
    /// Whether the check ran; it is left out after lexer or parser errors.
    pub checked: bool,
    /// The tree view of the IR as lowered.
    pub ir_plain: TreeNode,
    /// The tree view of the IR after folding.
    pub ir_folded: TreeNode,
    /// The S-expression form of the IR as lowered.
    pub ir_plain_sexpr: String,
    /// The S-expression form of the IR after folding.
    pub ir_folded_sexpr: String,
    /// Whether lowering ran; it is left out after errors in earlier phases.
    pub lowered: bool,
    /// Whether the ownership check ran; it is left out after errors in
    /// earlier phases, folding included.
    pub borrowchecked: bool,
    /// What each function frees, resizes or moves through, one line each.
    pub digests: Vec<String>,
    /// The generated ops, which the run view steps through.
    pub ops: Option<bc::Module>,
    /// The functions of the ops, as the asm tables show them.
    pub asm: Vec<AsmFunc>,
    /// The global slots of the ops.
    pub globals: Vec<SlotRow>,
    /// The static texts of the ops.
    pub texts: Vec<String>,
    /// The pieces of each text the ops build when the program runs, as
    /// listing text.
    pub formats: Vec<String>,
    /// Whether code generation ran; it is left out after errors in earlier
    /// phases, folding included.
    pub emitted: bool,
    /// How long each phase took.
    #[builder(default)]
    pub times: PhaseTimes,
}

/// Which tree a view shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSource
{
    /// The parser tree.
    Ast,
    /// The IR as lowered.
    Plain,
    /// The IR after folding.
    Folded,
}

impl TreeSource
{
    /// The tree in a compile run.
    ///
    /// # Arguments
    ///
    /// * `c` - The run.
    ///
    /// # Returns
    ///
    /// The tree.
    pub fn tree(self, c: &Compiled) -> &TreeNode
    {
        match self
        {
            TreeSource::Ast => &c.tree,
            TreeSource::Plain => &c.ir_plain,
            TreeSource::Folded => &c.ir_folded,
        }
    }

    /// The S-expression form in a compile run.
    ///
    /// # Arguments
    ///
    /// * `c` - The run.
    ///
    /// # Returns
    ///
    /// The text.
    pub fn sexpr(self, c: &Compiled) -> &str
    {
        match self
        {
            TreeSource::Ast => &c.sexpr,
            TreeSource::Plain => &c.ir_plain_sexpr,
            TreeSource::Folded => &c.ir_folded_sexpr,
        }
    }
}

impl Compiled
{
    /// The token ranges, for lookups with
    /// [`crate::compile::spans::find_range`].
    ///
    /// # Returns
    ///
    /// The ranges, in source order.
    pub fn token_ranges(&self) -> Vec<Range>
    {
        self.tokens.iter().map(|t| t.range).collect()
    }

    /// The token under a UTF-16 offset of the editor.
    ///
    /// # Arguments
    ///
    /// * `off16` - The UTF-16 offset.
    ///
    /// # Returns
    ///
    /// The token index, or `None` between tokens.
    pub fn token_at(&self, off16: u32) -> Option<usize>
    {
        let off = self.table.to_utf8(off16);
        find_in(&self.tokens, off, |t| t.range)
    }

    /// Count the diagnostics of a level.
    ///
    /// # Arguments
    ///
    /// * `level` - The level.
    ///
    /// # Returns
    ///
    /// The count.
    pub fn count(&self, level: Level) -> usize
    {
        self.diagnostics.iter().filter(|d| d.level == level).count()
    }

    /// How a phase went: the worst level among its diagnostics.
    ///
    /// # Arguments
    ///
    /// * `phase` - The phase.
    ///
    /// # Returns
    ///
    /// The status; the check is skipped after lexer or parser errors, and
    /// lowering, the ownership check, code generation and the run after
    /// errors in an earlier phase. The ops do not run here; the pipeline warns
    /// on the run once a run of the debugger session traps.
    pub fn status(&self, phase: Phase) -> Status
    {
        let emits = matches!(phase, Phase::Codegen | Phase::Run);
        ret_if!(phase == Phase::Check && !self.checked, Status::Skipped);
        ret_if!(phase == Phase::Lower && !self.lowered, Status::Skipped);
        ret_if!(phase == Phase::BorrowCheck && !self.borrowchecked, Status::Skipped);
        ret_if!(emits && !self.emitted, Status::Skipped);
        let own = self.diagnostics.iter().filter(|d| d.phase == phase);
        let levels: Vec<Level> = own.map(|d| d.level).collect();
        ret_if!(levels.contains(&Level::Error), Status::Failed);
        tern!(levels.contains(&Level::Warning), Status::Warned, Status::Clean)
    }

    /// The last phase of the run of phases from the first that passed, with
    /// or without warnings; the stage debug mode opens on.
    ///
    /// # Returns
    ///
    /// The phase before the first that failed or was skipped, or the last
    /// phase when all passed.
    pub fn last_passed(&self) -> Phase
    {
        let passed =
            |p: &Phase| matches!(self.status(*p), Status::Clean | Status::Warned);
        Phase::iter().take_while(passed).last().unwrap_or(Phase::Segment)
    }
}

/// The source map, file and text a compile run works on.
#[derive(Builder)]
struct Unit<'a>
{
    /// The source map that holds the file.
    sm: &'a SourceMap,
    /// The file.
    file: &'a SourceFile,
    /// The original text.
    original: &'a str,
}

impl Unit<'_>
{
    /// Range of a span in the original text.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The range.
    fn range(&self, span: &Span) -> Range
    {
        to_original(self.file, self.original, span)
    }

    /// Line and column of the start of a span.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The 1-based line and 1-based column.
    fn line_col(&self, span: &Span) -> (usize, usize)
    {
        let relative = self.file.relative_position(&span.lo);
        let (line, col) = self.file.lookup_file_pos(&relative);
        (line, col.as_u32() as usize + 1)
    }

    /// Normalized source text of a span.
    ///
    /// # Arguments
    ///
    /// * `span` - The span.
    ///
    /// # Returns
    ///
    /// The text.
    fn text(&self, span: &Span) -> String
    {
        let lo = self.file.relative_position(&span.lo);
        let hi = self.file.relative_position(&span.hi);
        self.file.snippet(&lo, &hi).unwrap_or("").to_string()
    }
}

/// Run all phases over `source`.
///
/// # Arguments
///
/// * `source` - The text the user wrote.
/// * `options` - The options.
///
/// # Returns
///
/// The rows and trees of all views.
pub fn compile(source: &str, options: &Options) -> Compiled
{
    let mut sm = SourceMap::new();
    let name = FileName::Custom(FILE_NAME.to_string());
    let file = sm.new_source_file(name, source.to_string());
    let unit = Unit::builder().sm(&sm).file(&file).original(source).build();

    let segment_options =
        SegmenterOptions::builder().coalesce_non_burmese_chars(false).build();
    let (segments, segment_time) =
        timed(|| segment_at(&file.src, file.start_pos, &segment_options));
    let (lexed, lex_time) = timed(|| lex(&file, &LexerOptions::default()));
    let (mut parsed, parse_time) = timed(|| parse(&file, &lexed));
    // The explorer holds one file, so it has no folder to import from.
    parsed.diagnostics.extend(refuse_imports(&parsed.program));
    let checked = !has_errors(&lexed.diagnostics) && !has_errors(&parsed.diagnostics);
    let ((mut idiom, out), check_time) = timed(|| {
        tern!(checked, check_program(&parsed.program, options), Default::default())
    });
    idiom.extend(out.diagnostics.iter().cloned());
    let lowered = checked && !has_errors(&idiom);
    let (ir, lower_time) =
        timed(|| lowered.then(|| lower_and_fold(&parsed.program, &out, options)));
    let (plain, mut folded, fold_diags) = ir.unwrap_or_default();
    let borrowchecked = lowered && !has_errors(&fold_diags);
    let ((owner_diags, digests), borrow_check_time) = timed(|| {
        let module = folded.as_mut().filter(|_| borrowchecked);
        module.map(|m| check_owners(m, options)).unwrap_or_default()
    });
    let tagged = [
        (Phase::Lex, &lexed.diagnostics),
        (Phase::Parse, &parsed.diagnostics),
        (Phase::Check, &idiom),
        (Phase::Lower, &fold_diags),
        (Phase::BorrowCheck, &owner_diags),
    ];
    let ir_tree = |m: &Option<Module>| {
        m.as_ref().map(|m| build_ir_tree(&file, source, m)).unwrap_or_default()
    };
    let ir_sexpr =
        |m: &Option<Module>| m.as_ref().map(|m| m.to_sexpr()).unwrap_or_default();
    let diagnostics = tagged
        .iter()
        .flat_map(|(phase, diags)| diags.iter().map(|d| diag_row(&unit, *phase, d)))
        .collect();
    let emitted = borrowchecked && !has_errors(&owner_diags);
    let (ops, codegen_time) = timed(|| {
        folded.as_ref().filter(|_| emitted).map(|m| {
            let emitted = emit(m, options.platform_bits, emit_options(options));
            bc::Module { overflow: options.overflow, ..emitted }
        })
    });
    let times = PhaseTimes::builder()
        .segment(segment_time)
        .lex(lex_time)
        .parse(parse_time)
        .maybe_check(checked.then_some(check_time))
        .maybe_lower(lowered.then_some(lower_time))
        .maybe_borrow_check(borrowchecked.then_some(borrow_check_time))
        .maybe_codegen(emitted.then_some(codegen_time))
        .build();
    let asm = ops.as_ref().map(|m| asm_rows(&unit, m)).unwrap_or_default();
    let globals = ops.as_ref().map(|m| slot_rows(&m.globals)).unwrap_or_default();
    let texts = ops.as_ref().map(|m| m.texts.clone()).unwrap_or_default();
    let formats =
        ops.as_ref().map(|m| m.formats.iter().map(|f| format_text(f)).collect());

    Compiled::builder()
        .source(source.to_string())
        .table(OffsetTable::new(source))
        .segments(segment_rows(&unit, &segments))
        .tokens(token_rows(&unit, &lexed.tokens))
        .comments(comment_rows(&unit, &lexed))
        .strings(string_rows(&unit, &lexed))
        .tree(build_tree(&file, source, &parsed.program, &out.kinds))
        .sexpr(parsed.program.to_sexpr())
        .diagnostics(diagnostics)
        .checked(checked)
        .ir_plain(ir_tree(&plain))
        .ir_folded(ir_tree(&folded))
        .ir_plain_sexpr(ir_sexpr(&plain))
        .ir_folded_sexpr(ir_sexpr(&folded))
        .lowered(lowered)
        .borrowchecked(borrowchecked)
        .digests(digests)
        .maybe_ops(ops)
        .asm(asm)
        .globals(globals)
        .texts(texts)
        .formats(formats.unwrap_or_default())
        .emitted(emitted)
        .times(times)
        .build()
}

/// Run the naturalness and the semantic checks over a parsed program.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `options` - The options.
///
/// # Returns
///
/// The naturalness diagnostics, and the semantic check output.
fn check_program(
    program: &zakac_ast::stmt::Program, options: &Options,
) -> (Vec<Diagnostic>, SemcheckOutput)
{
    let check_options = SemcheckOptions::builder()
        .platform_bits(options.platform_bits)
        .no_raw(options.no_raw)
        .build();
    let idiom = check(program, &NaturalnessOptions::default());
    (idiom, semcheck(program, &check_options))
}

/// The op budget of a run in the explorer, so a loop that never ends does
/// not hang the page.
pub const RUN_STEPS: u64 = 1_000_000;

/// The text of a trap, as the views show it.
///
/// # Arguments
///
/// * `trap` - The trap.
///
/// # Returns
///
/// The text.
pub fn trap_text(trap: &Trap) -> String
{
    t!("explorer.run.trapped", trap = trap).to_string()
}

/// Lower a checked program and fold a clone.
///
/// # Arguments
///
/// * `program` - The tree.
/// * `out` - The check output.
/// * `options` - The options.
///
/// # Returns
///
/// The module as lowered, the folded module and the fold diagnostics.
fn lower_and_fold(
    program: &zakac_ast::stmt::Program, out: &SemcheckOutput, options: &Options,
) -> (Option<Module>, Option<Module>, Vec<Diagnostic>)
{
    let bits = options.platform_bits;
    let plain = lower(
        program,
        &out.kinds,
        &out.structs,
        &LowerOptions::builder().platform_bits(bits).build(),
    );
    let mut folded = plain.clone();
    let diags = fold(&mut folded, &options.fold, bits);
    (Some(plain), Some(folded), diags)
}

/// Run the ownership check over a folded module, which marks the list cells
/// it proves in bounds.
///
/// # Arguments
///
/// * `module` - The folded module.
/// * `options` - The options.
///
/// # Returns
///
/// The diagnostics, and the digest of each function as one line.
fn check_owners(
    module: &mut Module, options: &Options,
) -> (Vec<Diagnostic>, Vec<String>)
{
    #[rustfmt::skip]
    let mem = BorrowCheckOptions::builder().check(options.check).strict(options.strict)
        .overflow(options.overflow);
    let out = borrow_check(module, &mem.build(), options.platform_bits);
    (out.diagnostics, out.digests.iter().map(ToString::to_string).collect())
}

/// Rows of the segments view.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `segments` - The segments.
///
/// # Returns
///
/// The rows.
fn segment_rows(unit: &Unit, segments: &[PreLexicalToken]) -> Vec<SegmentRow>
{
    let row = |seg: &PreLexicalToken| {
        let (line, col) = unit.line_col(&seg.span);
        SegmentRow::builder()
            .kind(seg.kind.into())
            .lexeme(seg.lexeme.to_string())
            .range(unit.range(&seg.span))
            .line(line)
            .col(col)
            .build()
    };
    segments.iter().map(row).collect()
}

/// Rows of the tokens view, without the end marker.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `tokens` - The lexer tokens.
///
/// # Returns
///
/// The rows.
fn token_rows(unit: &Unit, tokens: &[Token]) -> Vec<TokenRow>
{
    let live = tokens.iter().filter(|t| t.kind != TokenKind::Eof);
    live.map(|t| token_row(unit, t)).collect()
}

/// Row of one token.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `token` - The token.
///
/// # Returns
///
/// The row.
fn token_row(unit: &Unit, token: &Token) -> TokenRow
{
    let (line, col) = unit.line_col(&token.span);
    TokenRow::builder()
        .kind(token.kind.into())
        .detail(token_detail(&token.kind))
        .particle(particle_name(token.particle))
        .joined(token.joined)
        .lexeme(unit.text(&token.span))
        .range(unit.range(&token.span))
        .line(line)
        .col(col)
        .class(token_class(&token.kind))
        .build()
}

/// Detail of a token kind: the reserved word, operator, number shape or
/// decoded value. The form name comes from strum.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// The detail text, blank for identifiers.
pub fn token_detail(kind: &TokenKind) -> String
{
    match kind
    {
        TokenKind::Number(shape) => number_detail(shape),
        TokenKind::Char(ch) => format!("{ch:?}"),
        TokenKind::Str(idx) => format!("#{idx}"),
        TokenKind::Kw(kw) => format!("{kw:?}"),
        TokenKind::Punct(punct) => punct.as_ref().to_string(),
        TokenKind::Ident | TokenKind::RawIdent | TokenKind::Eof => String::new(),
    }
}

/// Detail text of a number shape.
///
/// # Arguments
///
/// * `shape` - The shape.
///
/// # Returns
///
/// The script and the parts that are present.
fn number_detail(shape: &zakac_lexer::token::NumberShape) -> String
{
    let flags = [
        (shape.has_fraction, "fraction"),
        (shape.has_exponent, "exponent"),
        (shape.has_letters, "letters"),
    ];
    let parts = flags.iter().filter(|(on, _)| *on).map(|(_, name)| *name);
    let script = format!("{:?}", shape.script).to_lowercase();
    std::iter::once(script.as_str()).chain(parts).collect::<Vec<_>>().join(" ")
}

/// Name of a particle; blank for none.
///
/// # Arguments
///
/// * `particle` - The particle.
///
/// # Returns
///
/// The name.
fn particle_name(particle: Particle) -> String
{
    tern!(particle == Particle::None, String::new(), format!("{particle:?}"))
}

/// Rows of the comments table.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `lexed` - The lexer output.
///
/// # Returns
///
/// The rows.
fn comment_rows(unit: &Unit, lexed: &LexOutput) -> Vec<CommentRow>
{
    let row = |c: &zakac_lexer::token::Comment| {
        CommentRow::builder()
            .kind(c.kind.into())
            .text(unit.text(&c.span))
            .range(unit.range(&c.span))
            .line(unit.line_col(&c.span).0)
            .build()
    };
    lexed.comments.iter().map(row).collect()
}

/// Rows of the strings table; each takes its source text and region from
/// its token.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `lexed` - The lexer output.
///
/// # Returns
///
/// The rows.
fn string_rows(unit: &Unit, lexed: &LexOutput) -> Vec<StrRow>
{
    let literal = |t: &Token| match t.kind
    {
        TokenKind::Str(idx) => Some((idx as usize, t.span)),
        _ => None,
    };
    let spans: Vec<(usize, Span)> = lexed.tokens.iter().filter_map(literal).collect();
    let row = |(idx, lit): (usize, &zakac_lexer::token::StrLit)| {
        let span = spans.iter().find(|(i, _)| *i == idx).map(|(_, s)| *s);
        StrRow::builder()
            .idx(idx)
            .value(lit.parts.iter().map(str_part).collect())
            .source(span.map(|s| unit.text(&s)).unwrap_or_default())
            .range(span.map(|s| unit.range(&s)).unwrap_or_default())
            .build()
    };
    lexed.strings.iter().enumerate().map(row).collect()
}

/// A string piece as the table shows it. Text keeps its chars unchanged;
/// Rust debug quoting would spell the Burmese marks as `\u{...}`.
///
/// # Arguments
///
/// * `part` - The piece.
///
/// # Returns
///
/// The piece.
fn str_part(part: &StrPart) -> StrPiece
{
    let (kind, text) = match part
    {
        StrPart::Text(text) => (PartKind::Text, text.clone()),
        StrPart::Escape(ch) => (PartKind::Escape, escape_controls(&ch.to_string())),
        StrPart::TemplateSlot { text, .. } =>
        {
            (PartKind::TemplateSlot, format!("{{{text}}}"))
        }
        StrPart::Bad(text) => (PartKind::Escape, text.clone()),
    };
    StrPiece::builder().kind(kind).text(text).build()
}

/// A string with its control chars written as escapes: `\n`, `\t`, `\r`
/// and `\0`, else `\u{...}`.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The text; other chars, Burmese marks included, stand unchanged.
pub fn escape_controls(text: &str) -> String
{
    let shown = |c: char| match c
    {
        '\n' => "\\n".to_string(),
        '\t' => "\\t".to_string(),
        '\r' => "\\r".to_string(),
        '\0' => "\\0".to_string(),
        c if c.is_control() => format!("\\u{{{:x}}}", u32::from(c)),
        c => c.to_string(),
    };
    text.chars().map(shown).collect()
}

/// Row of a diagnostic.
///
/// # Arguments
///
/// * `unit` - The compile unit.
/// * `phase` - The phase that reported it.
/// * `diag` - The diagnostic.
///
/// # Returns
///
/// The row.
fn diag_row(unit: &Unit, phase: Phase, diag: &Diagnostic) -> DiagRow
{
    let range = (!diag.span.is_placeholder()).then(|| unit.range(&diag.span));
    let label = |l: &Label| {
        (!l.span.is_placeholder()).then(|| (unit.range(&l.span), l.text.clone()))
    };
    DiagRow::builder()
        .phase(phase)
        .level(diag.level)
        .code(diag.code.to_string())
        .message(diag.message.clone())
        .maybe_help(diag.help.clone())
        .maybe_range(range)
        .labels(diag.labels.iter().filter_map(label).collect())
        .pieces(diagnostic_pieces(unit.sm, diag))
        .build()
}
