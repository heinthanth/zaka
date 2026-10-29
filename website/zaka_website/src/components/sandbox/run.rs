use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::MemCheck;
use zakac_codegen::artifact::run_phases;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_reporter::render::Paint;
use zakac_reporter::render::diagnostic_pieces;
use zakac_rust_sugars::do_while;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The name a program of the page gets in its diagnostics.
const FILE_NAME: &str = "main.zaka";

/// The most ops one run takes, so a loop that never ends lets the page go.
pub const STEP_CAP: u64 = 20_000_000;

/// The most ops a run takes in one batch; one batch runs a frame.
pub const BATCH: u64 = 100_000;

/// What a fence of a Learn page promises its program does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fence
{
    /// It runs to its end: `zaka`.
    Run,
    /// It fails to compile with these codes: `zaka-error ZK0103`.
    Error(Vec<String>),
    /// It compiles with these warnings: `zaka-warning ZK0501`.
    Warning(Vec<String>),
    /// It stops with an error while it runs: `zaka-stop`.
    Stop,
}

/// What a fence of a Learn page asks of the compiler besides its promise:
/// `mem-check=warn`, `mem-strict=on`, `no-raw=on`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Builder, Serialize, Deserialize)]
pub struct FenceOptions
{
    /// How the ownership check reports what it finds.
    #[builder(default)]
    pub check: MemCheck,
    /// Whether the check also reports the accesses it cannot prove safe.
    #[builder(default)]
    pub strict: bool,
    /// Whether raw addresses are refused.
    #[builder(default)]
    pub no_raw: bool,
}

impl FenceOptions
{
    /// Take one word of a fence, such as `mem-check=warn`.
    ///
    /// # Arguments
    ///
    /// * `word` - The word, with its equals sign.
    ///
    /// # Returns
    ///
    /// The options with the word taken in, or `None` for a word that names no
    /// option or gives it a value it does not take.
    fn with_word(mut self, word: &str) -> Option<Self>
    {
        let (name, value) = word.split_once('=')?;
        match (name, value)
        {
            ("mem-check", _) => self.check = value.parse().ok()?,
            ("mem-strict", "on") => self.strict = true,
            ("no-raw", "on") => self.no_raw = true,
            _ => return None,
        }
        Some(self)
    }
}

impl Fence
{
    /// Read the info string of a fence.
    ///
    /// # Arguments
    ///
    /// * `info` - The info string, such as `zaka-error ZK0103`; its words with
    ///   an equals sign are options, the others codes.
    ///
    /// # Returns
    ///
    /// The fence and its options, or `None` for a fence that holds no Zaka or
    /// names an option it does not have.
    pub fn parse(info: &str) -> Option<(Fence, FenceOptions)>
    {
        let mut words = info.split_whitespace();
        let head = words.next()?;
        let (settings, codes): (Vec<&str>, Vec<&str>) =
            words.partition(|w| w.contains('='));
        ret_if!(!codes.iter().all(|w| is_code(w)), None);
        let codes = || codes.iter().map(|c| c.to_string()).collect();
        let fence = match head
        {
            "zaka" => Fence::Run,
            "zaka-error" => Fence::Error(codes()),
            "zaka-warning" => Fence::Warning(codes()),
            "zaka-stop" => Fence::Stop,
            _ => return None,
        };
        let none = FenceOptions::default();
        let options = settings.into_iter().try_fold(none, FenceOptions::with_word)?;
        Some((fence, options))
    }

    /// The codes the fence names.
    ///
    /// # Returns
    ///
    /// The codes; none for a fence that names none.
    pub fn codes(&self) -> &[String]
    {
        match self
        {
            Fence::Error(codes) | Fence::Warning(codes) => codes,
            Fence::Run | Fence::Stop => &[],
        }
    }
}

/// Whether a fence word is a diagnostic code: `ZK` and four digits.
///
/// # Arguments
///
/// * `word` - The word.
///
/// # Returns
///
/// Whether it is a code.
fn is_code(word: &str) -> bool
{
    let digits = word.strip_prefix("ZK").unwrap_or_default();
    digits.len() == 4 && digits.chars().all(|c| c.is_ascii_digit())
}

/// A run of rendered text and how to color it.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Stroke
{
    /// The text; it can end in a line break.
    #[builder(into)]
    pub text: String,
    /// How to color it.
    pub tone: Tone,
}

/// How a run of output is colored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone
{
    /// What the program printed.
    Plain,
    /// The gutter of a diagnostic.
    Gutter,
    /// An error: a tag, its marked source, or the reason a run stopped.
    Error,
    /// A warning.
    Warning,
    /// A note.
    Note,
}

/// What compiling a program gives: its diagnostics, rendered, and the
/// machine that runs it, when it compiles.
#[derive(Builder)]
pub struct Compiled
{
    /// The diagnostics, as the command line renders them.
    pub report: Vec<Stroke>,
    /// The codes of the diagnostics, in order.
    pub codes: Vec<String>,
    /// The machine, or `None` when an error stops the build.
    pub machine: Option<Machine>,
}

/// Compile a program and set a machine at its start.
///
/// # Arguments
///
/// * `source` - The program.
/// * `options` - What its fence asks of the compiler.
///
/// # Returns
///
/// The diagnostics and the machine.
pub fn compile(source: &str, options: &FenceOptions) -> Compiled
{
    let mut sm = SourceMap::new();
    let file =
        sm.new_source_file(FileName::Custom(FILE_NAME.into()), source.to_string());
    let emitted = run_phases(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::builder().no_raw(options.no_raw).build(),
        &FoldOptions::default(),
        &BorrowCheckOptions::builder()
            .check(options.check)
            .strict(options.strict)
            .build(),
    );
    let diags = emitted.diagnostics();
    let report = diags.iter().flat_map(|d| strokes(&sm, d)).collect();
    let codes = diags.iter().map(|d| d.code.to_string()).collect();
    let options = RunOptions::builder().max_steps(STEP_CAP).build();
    let machine = emitted.module.map(|m| Machine::new(m, options));
    Compiled::builder().report(report).codes(codes).maybe_machine(machine).build()
}

/// Run a machine for a batch of ops at most.
///
/// # Arguments
///
/// * `machine` - The machine.
/// * `ops` - The most ops to run.
///
/// # Returns
///
/// What it printed, and the reason it stopped when it stopped with an error.
pub fn advance(machine: &mut Machine, ops: u64) -> (String, Option<String>)
{
    let end = machine.steps().saturating_add(ops);
    do_while!(machine.running() && machine.steps() < end, _ = machine.step());
    let reason = |kind: String| t!("website.sandbox.stopped", e = kind).to_string();
    let stopped = machine.state().trap().map(|trap| reason(trap.kind.to_string()));
    (machine.take_output(), stopped)
}

/// Run a program to its end, or to the step cap.
///
/// # Arguments
///
/// * `source` - The program.
/// * `options` - What its fence asks of the compiler.
///
/// # Returns
///
/// The codes of its diagnostics, and its output as its page writes it: what
/// it printed, then the reason it stopped when it stopped with an error.
pub fn run_to_end(source: &str, options: &FenceOptions) -> (Vec<String>, String)
{
    let compiled = compile(source, options);
    let Some(mut machine) = compiled.machine
    else
    {
        return (compiled.codes, String::new());
    };
    let (printed, stopped) = advance(&mut machine, STEP_CAP);
    let stopped = stopped.map(|s| format!("{s}\n")).unwrap_or_default();
    (compiled.codes, format!("{printed}{stopped}"))
}

/// The strokes of one diagnostic, as the command line renders it.
///
/// # Arguments
///
/// * `sm` - The source map of its file.
/// * `diag` - The diagnostic.
///
/// # Returns
///
/// The strokes.
fn strokes(sm: &SourceMap, diag: &Diagnostic) -> Vec<Stroke>
{
    let tone = |paint: Paint| match paint
    {
        Paint::Plain => Tone::Plain,
        Paint::Gutter => Tone::Gutter,
        Paint::Tag(level) | Paint::Marked(level) | Paint::Carets(level) =>
        {
            level_tone(level)
        }
    };
    let pieces = diagnostic_pieces(sm, diag).into_iter();
    pieces.map(|p| Stroke::builder().tone(tone(p.paint)).text(p.text).build()).collect()
}

/// The tone of a diagnostic level.
///
/// # Arguments
///
/// * `level` - The level.
///
/// # Returns
///
/// The tone.
fn level_tone(level: Level) -> Tone
{
    match level
    {
        Level::Error => Tone::Error,
        Level::Warning => Tone::Warning,
        Level::Note => Tone::Note,
    }
}
