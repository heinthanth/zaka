//! The `zakac repl` command: read statements, run them and print their output.

use std::io::IsTerminal;
use std::path::PathBuf;

use etcetera::BaseStrategy;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use zaka_host::Gate;
use zaka_host::Grants;
use zaka_host::MemWorld;
use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_in;
use zaka_vm::trap::Backtrace;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_cli_common::clap;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_codegen_target_vm::artifact::run_phases;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_i18n::t;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_parser::parser::parse;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The name of the program a session builds, as messages show it.
const FILE_NAME: &str = "<repl>";

/// The `zakac repl` command.
pub(crate) struct ZakacRepl
{
    /// The clap command.
    cmd_instance: clap::Command,
}

/// The chunks a session kept and the output it has shown.
// shortcut: each chunk runs the whole session again and prints the new tail of
// its output, in a world held in storage (no stdin, no disk); keep the VM
// between chunks when a program that prints a new value on each run matters.
#[derive(Default)]
struct Session
{
    /// The chunks that compiled and ran, in order.
    kept: String,
    /// The octets of output shown so far.
    shown: usize,
}

impl Session
{
    /// Whether an chunk leaves a block open: the session with it fails to
    /// parse at its end, so more lines are to come.
    ///
    /// # Arguments
    ///
    /// * `chunk` - The lines of the chunk so far.
    ///
    /// # Returns
    ///
    /// `true` while the chunk is not whole.
    fn is_open(&self, chunk: &str) -> bool
    {
        let text = format!("{}{chunk}", self.kept);
        let end = text.trim_end().len() as u32;
        let mut sm = SourceMap::new();
        let file = sm.new_source_file(FileName::Custom(FILE_NAME.into()), text);
        let lexed = lex(&file, &LexerOptions::default());
        let parsed = parse(&file, &lexed);
        let base = file.start_pos.as_u32();
        let mut diags = lexed.diagnostics.iter().chain(&parsed.diagnostics);
        diags.any(|d| d.span.hi.as_u32() - base >= end)
    }

    /// Compile the session with an chunk and run it: print the diagnostics,
    /// then the output past what was shown. The chunk is kept when it
    /// compiles and runs to its end.
    ///
    /// # Arguments
    ///
    /// * `chunk` - The chunk.
    /// * `io` - The standard streams.
    fn take(&mut self, chunk: &str, io: &mut CommandIO<'_>)
    {
        ret_if!(chunk.trim().is_empty());
        let text = format!("{}{chunk}", self.kept);
        let mut sm = SourceMap::new();
        let file = sm.new_source_file(FileName::Custom(FILE_NAME.into()), text.clone());
        let options = SemcheckOptions::builder().repl(true).build();
        #[rustfmt::skip]
        let emitted = run_phases(&file, &LexerOptions::default(), &options,
            &FoldOptions::default(), &BorrowCheckOptions::default(), ResultMode::default());
        // The earlier chunks compiled: their warnings were shown once.
        let new_from = file.start_pos.as_u32() + self.kept.len() as u32;
        let shown = emitted.diagnostics().iter();
        let diags: Vec<_> = shown
            .filter(|d| d.is_error() || d.span.lo.as_u32() >= new_from)
            .cloned()
            .collect();
        let _ = write_diagnostics(io.stderr, &sm, &diags);
        let Some(module) = emitted.module
        else
        {
            return;
        };
        let mut out = Vec::new();
        let world = Box::new(MemWorld::builder().build());
        let gate = Gate::new(Grants { ffi: false, ..Grants::all() });
        let host = HostData::program(FILE_NAME);
        let ran = run_in(&module, &mut out, &RunOptions::default(), &host, world, gate);
        let _ = io.stdout.write_all(out.get(self.shown..).unwrap_or_default());
        let _ = io.stdout.flush();
        let Err(trap) = ran.1
        else
        {
            self.kept = text;
            self.shown = out.len();
            return;
        };
        let _ = writeln!(io.stderr, "{}", trap.report(&sm, Backtrace::default()));
    }
}

/// What a read of one line gives.
enum Got
{
    /// A line, with no line break.
    Line(String),
    /// Ctrl-C: the lines of the chunk so far are dropped.
    Cancel,
    /// The end of the input: Ctrl-D, or the end of a pipe.
    End,
}

/// Where the lines of a session come from: a terminal, through a line editor
/// with the lines of earlier sessions, or the plain standard input of a pipe
/// or a file.
enum Lines
{
    /// A terminal.
    Term(DefaultEditor),
    /// A pipe or a file.
    Plain,
}

/// The file that keeps the lines of the sessions on a terminal.
///
/// # Returns
///
/// `zakac/repl_lines` in the data folder of the user, or `None` when the user
/// has none.
fn lines_file() -> Option<PathBuf>
{
    let base = etcetera::choose_base_strategy().ok()?;
    Some(base.data_dir().join("zakac").join("repl_lines"))
}

impl Lines
{
    /// The line editor on a terminal, with the lines of earlier sessions;
    /// plain lines otherwise.
    ///
    /// # Returns
    ///
    /// The lines.
    fn open() -> Self
    {
        ret_if!(!std::io::stdin().is_terminal(), Lines::Plain);
        let Ok(mut editor) = DefaultEditor::new()
        else
        {
            return Lines::Plain;
        };
        // A first session has no file of earlier lines.
        let _ = lines_file().map(|f| editor.load_history(&f));
        Lines::Term(editor)
    }

    /// Read a line.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt.
    /// * `io` - The standard streams, for plain lines.
    ///
    /// # Returns
    ///
    /// The line, or what ended it.
    fn read(&mut self, prompt: &str, io: &mut CommandIO<'_>) -> Got
    {
        let Lines::Term(editor) = self
        else
        {
            return plain_line(prompt, io);
        };
        match editor.readline(prompt)
        {
            Ok(line) =>
            {
                do_if!(!line.trim().is_empty(), _ = editor.add_history_entry(&line));
                Got::Line(line)
            }
            Err(ReadlineError::Interrupted) => Got::Cancel,
            Err(_) => Got::End,
        }
    }

    /// Clear the screen of a terminal.
    fn clear(&mut self)
    {
        do_if!(let Lines::Term(editor) = self, _ = editor.clear_screen());
    }

    /// Keep the lines of the session on a terminal for the next one.
    fn save(&mut self)
    {
        let Lines::Term(editor) = self
        else
        {
            return;
        };
        let Some(file) = lines_file()
        else
        {
            return;
        };
        let _ = file.parent().map(std::fs::create_dir_all);
        let _ = editor.save_history(&file);
    }
}

/// Read a line of plain standard input, after a prompt on stderr.
///
/// # Arguments
///
/// * `prompt` - The prompt.
/// * `io` - The standard streams.
///
/// # Returns
///
/// The line, or [`Got::End`] at the end of the input.
fn plain_line(prompt: &str, io: &mut CommandIO<'_>) -> Got
{
    let _ = write!(io.stderr, "{prompt}");
    let _ = io.stderr.flush();
    let mut line = String::new();
    let read = io.stdin.read_line(&mut line).unwrap_or(0);
    ret_if!(read == 0, Got::End);
    Got::Line(line.trim_end_matches(['\n', '\r']).to_string())
}

/// Read one chunk: a line, and the lines after it while a block is open. A
/// blank line ends a chunk as it stands, and a line that starts with `:` is
/// a command, which stands alone.
///
/// # Arguments
///
/// * `session` - The session.
/// * `lines` - Where the lines come from.
/// * `io` - The standard streams.
///
/// # Returns
///
/// The chunk, or `None` at the end of the input.
fn read_chunk(
    session: &Session, lines: &mut Lines, io: &mut CommandIO<'_>,
) -> Option<String>
{
    let mut chunk = String::new();
    loop
    {
        let line = match lines.read(tern!(chunk.is_empty(), "> ", "... "), io)
        {
            Got::Line(line) => line,
            Got::Cancel =>
            {
                chunk.clear();
                continue;
            }
            Got::End => return (!chunk.is_empty()).then_some(chunk),
        };
        ret_if!(chunk.is_empty() && line.trim_start().starts_with(':'), Some(line));
        chunk.push_str(&line);
        chunk.push('\n');
        ret_if!(line.trim().is_empty() || !session.is_open(&chunk), Some(chunk));
    }
}

/// Run a session until the input ends or `:exit`. `:clear` clears the
/// screen and `:reset` forgets the statements so far.
///
/// # Arguments
///
/// * `io` - The standard streams.
///
/// # Returns
///
/// [`ExitCode::Success`].
fn command_main(io: &mut CommandIO<'_>) -> ExitCode
{
    let _ = writeln!(io.stderr, "{}", t!("repl.welcome"));
    let mut session = Session::default();
    let mut lines = Lines::open();
    while let Some(chunk) = read_chunk(&session, &mut lines, io)
    {
        match chunk.trim()
        {
            ":exit" => break,
            ":clear" => lines.clear(),
            ":reset" =>
            {
                session = Session::default();
                let _ = writeln!(io.stderr, "{}", t!("repl.reset"));
            }
            word if word.starts_with(':') =>
            {
                let _ = writeln!(io.stderr, "{}", t!("repl.unknown", c = word));
            }
            _ => session.take(&chunk, io),
        }
    }
    lines.save();
    ExitCode::Success
}

impl Command for ZakacRepl
{
    /// Get the canonical name of the command.
    ///
    /// # Returns
    ///
    /// The canonical name of the command.
    fn name() -> &'static str { "repl" }

    /// Get the name of the command when run on its own.
    ///
    /// # Returns
    ///
    /// The name.
    fn standalone_name() -> &'static str { Self::name() }

    /// Get the aliases of the command.
    ///
    /// # Returns
    ///
    /// The aliases.
    fn aliases() -> &'static [&'static str] { &[] }

    /// Get the version of the command.
    ///
    /// # Returns
    ///
    /// The version.
    fn version() -> &'static str { clap::crate_version!() }

    /// Configure the command.
    ///
    /// # Arguments
    ///
    /// * `setup` - The setup to configure.
    /// * `_mode` - The mode the command runs in.
    ///
    /// # Returns
    ///
    /// The configured setup.
    fn configure(setup: CommandSetup, _mode: CommandMode) -> CommandSetup
    {
        setup.with(|cmd| cmd.about(t!("repl.about")))
    }

    /// Build the command from its setup.
    ///
    /// # Arguments
    ///
    /// * `setup` - The setup.
    ///
    /// # Returns
    ///
    /// The command.
    fn from_setup(setup: CommandSetup) -> Self
    {
        let (cmd_instance, _) = setup.into_parts();
        Self { cmd_instance }
    }

    /// Get the clap command.
    ///
    /// # Returns
    ///
    /// The clap command.
    fn clap_command(&self) -> &clap::Command { &self.cmd_instance }

    /// Run the command.
    ///
    /// # Arguments
    ///
    /// * `_matches` - The matches of the CLI arguments.
    /// * `io` - The standard streams.
    ///
    /// # Returns
    ///
    /// The exit code.
    fn main(&self, _matches: &ArgMatches, io: &mut CommandIO<'_>) -> ExitCode
    {
        command_main(io)
    }
}
