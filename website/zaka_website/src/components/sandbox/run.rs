use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zakac_codegen::artifact::run_phases;
use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::Level;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_reporter::render::Paint;
use zakac_reporter::render::diagnostic_pieces;
use zakac_rust_sugars::do_while;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The name a program of the page gets in its diagnostics.
const FILE_NAME: &str = "main.zaka";

/// The most ops one run takes, so a loop that never ends lets the page go.
pub const STEP_CAP: u64 = 20_000_000;

/// The most ops a run takes in one batch; one batch runs a frame.
pub const BATCH: u64 = 100_000;

/// What a fence of the manual promises its program does.
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

impl Fence
{
    /// Read the info string of a fence.
    ///
    /// # Arguments
    ///
    /// * `info` - The info string, such as `zaka-error ZK0103`.
    ///
    /// # Returns
    ///
    /// The fence, or `None` for a fence that holds no Zaka.
    pub fn parse(info: &str) -> Option<Fence>
    {
        let mut words = info.split_whitespace();
        let codes =
            |words: std::str::SplitWhitespace| words.map(String::from).collect();
        match words.next()?
        {
            "zaka" => Some(Fence::Run),
            "zaka-error" => Some(Fence::Error(codes(words))),
            "zaka-warning" => Some(Fence::Warning(codes(words))),
            "zaka-stop" => Some(Fence::Stop),
            _ => None,
        }
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
///
/// # Returns
///
/// The diagnostics and the machine.
pub fn compile(source: &str) -> Compiled
{
    let mut sm = SourceMap::new();
    let file =
        sm.new_source_file(FileName::Custom(FILE_NAME.into()), source.to_string());
    let emitted = run_phases(
        &file,
        &LexerOptions::default(),
        &SemcheckOptions::default(),
        &FoldOptions::default(),
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
///
/// # Returns
///
/// The codes of its diagnostics, and its output as the manual writes it: what
/// it printed, then the reason it stopped when it stopped with an error.
pub fn run_to_end(source: &str) -> (Vec<String>, String)
{
    let compiled = compile(source);
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
