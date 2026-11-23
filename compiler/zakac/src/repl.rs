//! The `zakac repl` command: read statements, run them and print their output.

use std::io::IsTerminal;
use std::path::PathBuf;

use etcetera::BaseStrategy;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use zaka_host::Gate;
use zaka_host::Grants;
use zaka_host::MemWorld;
use zaka_host::World;
use zaka_vm::machine::HostData;
use zaka_vm::machine::Machine;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::drive;
use zaka_vm::machine::run_in;
use zaka_vm::trap::Backtrace;
use zakac_ast::stmt::START_NAME;
use zakac_borrow_checker::borrow_check;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_cli_common::clap;
use zakac_cli_common::clap::ArgMatches;
use zakac_cli_common::command::Command;
use zakac_cli_common::command::CommandIO;
use zakac_cli_common::command::CommandMode;
use zakac_cli_common::command::CommandSetup;
use zakac_cli_common::command::ExitCode;
use zakac_codegen_target_vm::artifact::Emitted;
use zakac_codegen_target_vm::artifact::run_phases;
use zakac_codegen_target_vm::bc::Module;
use zakac_codegen_target_vm::emit::emit;
use zakac_codegen_target_vm::options::ResultMode;
use zakac_i18n::t;
use zakac_ir_builder::fold::fold;
use zakac_ir_builder::ir;
use zakac_ir_builder::ir::FoldOptions;
use zakac_ir_builder::ir::LowerOptions;
use zakac_ir_builder::lower::lower;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_parser::parser::parse;
use zakac_reporter::render::write_diagnostics;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::artifact::Checked;
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

/// The chunks a session kept, the output it has shown, and the VM that ran
/// them, which goes on with each new chunk.
#[derive(Default)]
struct Session
{
    /// The chunks that compiled and ran, in order.
    kept: String,
    /// The octets of output shown so far.
    shown: usize,
    /// The VM, where the last chunk left it.
    machine: Option<Machine>,
    /// The globals of the module the VM ran last, as lowered.
    slots: Vec<ir::Slot>,
}

/// A compile of the session with a chunk.
struct Compiled
{
    /// The source map, for the reports.
    sm: SourceMap,
    /// Where the session text starts in the source map.
    base: u32,
    /// The output of each phase.
    emitted: Emitted,
}

impl Session
{
    /// Whether the session with a chunk fails to parse, and if so, whether
    /// at its end.
    ///
    /// # Arguments
    ///
    /// * `chunk` - The lines of the chunk so far.
    ///
    /// # Returns
    ///
    /// `None` for a clean parse, else whether an error reaches the end.
    fn parse_fails(&self, chunk: &str) -> Option<bool>
    {
        let text = format!("{}{chunk}", self.kept);
        let end = text.trim_end().len() as u32;
        let mut sm = SourceMap::new();
        let file = sm.new_source_file(FileName::Custom(FILE_NAME.into()), text);
        let lexed = lex(&file, &LexerOptions::default());
        let parsed = parse(&file, &lexed);
        let base = file.start_pos.as_u32();
        let diags: Vec<_> =
            lexed.diagnostics.iter().chain(&parsed.diagnostics).collect();
        ret_if!(diags.is_empty(), None);
        Some(diags.iter().any(|d| d.span.hi.as_u32() - base >= end))
    }

    /// Whether an chunk leaves a block open: the session with it fails to
    /// parse at its end, so more lines are to come. A chunk that is one
    /// expression is whole, since its value is printed.
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
        let whole = |grouped: bool| self.parse_fails(&echoed(chunk, grouped)).is_none();
        self.parse_fails(chunk) == Some(true) && !whole(false) && !whole(true)
    }

    /// Compile the session with a chunk, through each phase.
    ///
    /// # Arguments
    ///
    /// * `chunk` - The chunk.
    ///
    /// # Returns
    ///
    /// The compile.
    fn compile(&self, chunk: &str) -> Compiled
    {
        let text = format!("{}{chunk}", self.kept);
        let mut sm = SourceMap::new();
        let file = sm.new_source_file(FileName::Custom(FILE_NAME.into()), text);
        let options = SemcheckOptions::builder().repl(true).build();
        #[rustfmt::skip]
        let emitted = run_phases(&file, &LexerOptions::default(), &options,
            &FoldOptions::default(), &BorrowCheckOptions::default(), ResultMode::default());
        let base = file.start_pos.as_u32();
        Compiled { sm, base, emitted }
    }

    /// Compile the session with a chunk and run the chunk: print the
    /// diagnostics, then its output. A chunk that is one expression prints its
    /// value. The chunk is kept when it compiles and runs to its end.
    ///
    /// # Arguments
    ///
    /// * `chunk` - The chunk.
    /// * `io` - The standard streams.
    fn take(&mut self, chunk: &str, io: &mut CommandIO<'_>)
    {
        ret_if!(chunk.trim().is_empty());
        let first = self.compile(chunk);
        let failed =
            |c: &Compiled| c.emitted.diagnostics().iter().any(|d| d.is_error());
        // A chunk that is one expression prints its value; the parentheses
        // are for an expression that needs them.
        let echoes = [false, true].map(|grouped| echoed(chunk, grouped));
        let tried = echoes.iter().filter(|_| failed(&first));
        let echo =
            tried.map(|e| (e.as_str(), self.compile(e))).find(|(_, c)| !failed(c));
        let (chunk, compiled) = echo.unwrap_or((chunk, first));
        // The earlier chunks compiled: their warnings were shown once.
        let from = compiled.base + self.kept.len() as u32;
        let shown = compiled.emitted.diagnostics().iter();
        let diags: Vec<_> = shown
            .filter(|d| d.is_error() || d.span.lo.as_u32() >= from)
            .cloned()
            .collect();
        let _ = write_diagnostics(io.stderr, &compiled.sm, &diags);
        ret_if!(compiled.emitted.module.is_none());
        let checked = &compiled.emitted.borrowchecked.lowered.checked;
        let ran = self.go_on(checked, from, &compiled.sm, io);
        let ran = ran
            .unwrap_or_else(|| self.run_all(checked, compiled.base, &compiled.sm, io));
        do_if!(ran, self.kept.push_str(chunk));
    }

    /// Run the chunk on the kept VM, when the module of the session extends
    /// the one the VM ran last: each global from earlier chunks has its
    /// match, and each function keeps its number.
    ///
    /// # Arguments
    ///
    /// * `checked` - The checked session.
    /// * `from` - Where the chunk starts.
    /// * `sm` - The source map, for a report.
    /// * `io` - The standard streams.
    ///
    /// # Returns
    ///
    /// `None` when the VM cannot go on, else whether the chunk ran to its end;
    /// a chunk that traps leaves the VM as it was.
    fn go_on(
        &mut self, checked: &Checked, from: u32, sm: &SourceMap, io: &mut CommandIO<'_>,
    ) -> Option<bool>
    {
        let machine = self.machine.as_ref()?;
        let (slots, module) = session_module(checked, from);
        let end = checked.program.span.hi.as_u32();
        let kept = reuse(&self.slots, &slots, (from, end))?;
        ret_if!(!same_fns(machine.module(), &module), None);
        let mut next = machine.clone();
        next.go_on(module, &kept);
        next.set_host(&HostData::program(FILE_NAME));
        let mut out = Vec::new();
        let ran = drive(&mut next, &mut out, &mut |_, _| {});
        let _ = io.stdout.write_all(&out);
        let _ = io.stdout.flush();
        if let Err(trap) = ran
        {
            let _ = writeln!(io.stderr, "{}", trap.report(sm, Backtrace::default()));
            return Some(false);
        }
        self.shown += out.len();
        (self.machine, self.slots) = (Some(next), slots);
        Some(true)
    }

    /// Run the whole session on a new VM, and print the output past what was
    /// shown: for the first chunk, and for a module the kept VM cannot run.
    ///
    /// # Arguments
    ///
    /// * `checked` - The checked session.
    /// * `base` - Where the session text starts.
    /// * `sm` - The source map, for a report.
    /// * `io` - The standard streams.
    ///
    /// # Returns
    ///
    /// Whether the run reached its end.
    fn run_all(
        &mut self, checked: &Checked, base: u32, sm: &SourceMap, io: &mut CommandIO<'_>,
    ) -> bool
    {
        let (slots, module) = session_module(checked, base);
        let mut out = Vec::new();
        let world: Box<dyn World> = Box::new(MemWorld::builder().build());
        let reach = (world, Gate::new(Grants { ffi: false, ..Grants::all() }));
        let host = HostData::program(FILE_NAME);
        let options = RunOptions::default();
        let (machine, ran) =
            run_in(&module, &mut out, &options, &host, reach, &mut |_, _| {});
        let _ = io.stdout.write_all(out.get(self.shown..).unwrap_or_default());
        let _ = io.stdout.flush();
        if let Err(trap) = ran
        {
            let _ = writeln!(io.stderr, "{}", trap.report(sm, Backtrace::default()));
            return false;
        }
        self.shown = out.len();
        (self.machine, self.slots) = (Some(machine), slots);
        true
    }
}

/// A chunk as a print of its value, for a chunk that is one expression.
///
/// # Arguments
///
/// * `chunk` - The chunk.
/// * `grouped` - Whether the expression stands in parentheses.
///
/// # Returns
///
/// The print statement.
fn echoed(chunk: &str, grouped: bool) -> String
{
    let expr = chunk.trim().trim_end_matches('\u{104B}').trim_end();
    let expr = tern!(grouped, format!("({expr})"), expr.to_string());
    format!(
        "{expr} \u{1000}\u{102D}\u{102F} \
         \u{1015}\u{103C}\u{101E}\u{1019}\u{100A}\u{103A}\u{104B}\n"
    )
}

/// The module that runs a chunk of a session: the statements before `from`
/// ran in earlier chunks and leave no code, the variables of the start block
/// live in globals, and no global is dropped at the end.
///
/// # Arguments
///
/// * `checked` - The checked session.
/// * `from` - Where the chunk starts.
///
/// # Returns
///
/// The globals as lowered, and the module.
fn session_module(checked: &Checked, from: u32) -> (Vec<ir::Slot>, Module)
{
    let bits = SemcheckOptions::default().platform_bits;
    let lowering = LowerOptions::builder().platform_bits(bits).repl_from(from).build();
    let mut module =
        lower(&checked.program, &checked.kinds, &checked.structs, &lowering);
    // A constant set in an earlier chunk has no set in this module to fold.
    let folding = FoldOptions::builder().consts(false).build();
    let _ = fold(&mut module, &folding, bits);
    let mem = BorrowCheckOptions::builder().repl(true).build();
    let _ = borrow_check(&mut module, &mem, bits);
    let ops = Module {
        overflow: mem.overflow,
        ..emit(&module, bits, ResultMode::default().into())
    };
    (module.globals, ops)
}

/// Which old global each global of a new module takes over: a variable
/// declared before the chunk takes over the one of the same name, place and
/// kind; a global of an imported file, the one of the same name and kind at
/// the same distance past the session text, since the code that sets it ran
/// with the first chunk; each other global, a hidden one among them, gets a
/// block of its own.
///
/// # Arguments
///
/// * `old` - The globals of the module the VM ran last.
/// * `new` - The globals of the new module.
/// * `text` - Where the chunk starts, which is where the session text of the
///   old module ended, and where the session text ends now.
///
/// # Returns
///
/// The old global of each new one, or `None` when a variable from before the
/// chunk or a global of an imported file has no match, so the VM cannot go on.
fn reuse(
    old: &[ir::Slot], new: &[ir::Slot], text: (u32, u32),
) -> Option<Vec<Option<u32>>>
{
    let (from, end) = text;
    let imported = |s: &ir::Slot| s.span.lo.as_u32() >= end;
    let earlier = |s: &ir::Slot| s.span.lo.as_u32() < from && !s.name.starts_with('%');
    // A global of an imported file moves on as far as the session text grew.
    let place = |s: &ir::Slot| {
        let lo = s.span.lo.as_u32();
        tern!(
            imported(s),
            (lo - end, s.span.hi.as_u32() - end),
            (lo, s.span.hi.as_u32())
        )
    };
    let was = |o: &ir::Slot| {
        let lo = o.span.lo.as_u32();
        tern!(
            lo >= from,
            (lo - from, o.span.hi.as_u32() - from),
            (lo, o.span.hi.as_u32())
        )
    };
    let same = |s: &ir::Slot, o: &ir::Slot| {
        (&o.name, was(o), o.kind) == (&s.name, place(s), s.kind)
            && (o.span.lo.as_u32() >= from) == imported(s)
    };
    let take = |s: &ir::Slot| {
        let found = old.iter().position(|o| same(s, o)).map(|i| i as u32);
        tern!(earlier(s) || imported(s), found.map(Some), Some(None))
    };
    new.iter().map(take).collect()
}

/// Whether each function of the old module keeps its number and name in the
/// new one, since function values and closures hold the numbers; the main
/// function and the start block are run anew.
///
/// # Arguments
///
/// * `old` - The module the VM ran last.
/// * `new` - The new module.
///
/// # Returns
///
/// `true` if each does.
fn same_fns(old: &Module, new: &Module) -> bool
{
    let named = |i: usize, name: &str| new.funcs.get(i).is_some_and(|f| f.name == name);
    let run_anew = |i: usize, name: &str| i == old.main as usize || name == START_NAME;
    let funcs = old.funcs.iter().enumerate();
    funcs.filter(|(i, f)| !run_anew(*i, &f.name)).all(|(i, f)| named(i, &f.name))
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

#[cfg(test)]
mod tests
{
    use zakac_semcheck::kind::DataKind;
    use zakac_span::pos::Pos;
    use zakac_span::span::Span;

    use super::*;

    /// A global slot of a name, at a place.
    fn slot(name: &str, at: u32) -> ir::Slot
    {
        let span = Span::new(Pos::new(at), Pos::new(at + 1));
        #[rustfmt::skip]
        return ir::Slot::builder().name(name).kind(DataKind::Bool).access(ir::Access::Var)
            .span(span).build();
    }

    #[test]
    fn a_variable_from_an_earlier_chunk_takes_over_its_old_block()
    {
        let old = [slot("က", 0), slot("%new", 2), slot("ခ", 4)];
        let new = [slot("ခ", 4), slot("%new", 2), slot("က", 0), slot("ဂ", 9)];
        // A hidden global never shares a block; one from the new chunk gets
        // its own.
        assert_eq!(
            reuse(&old, &new, (8, 12)),
            Some(vec![Some(2), None, Some(0), None])
        );
    }

    #[test]
    fn a_variable_from_an_earlier_chunk_with_no_match_stops_the_vm()
    {
        let old = [slot("က", 0)];
        let new = [slot("က", 0), slot("ခ", 4)];
        assert_eq!(reuse(&old, &new, (8, 12)), None);
    }

    /// The session text grew from 8 to 12 octets: a global of an imported
    /// file moves on 4 octets and takes over its old block, and one with no
    /// match, from a file the chunk imports, stops the VM.
    #[test]
    fn a_global_of_an_imported_file_takes_over_its_old_block()
    {
        let old = [slot("က", 0), slot("ပ", 10)];
        let new = [slot("က", 0), slot("ပ", 14)];
        assert_eq!(reuse(&old, &new, (8, 12)), Some(vec![Some(0), Some(1)]));
        let added = [slot("က", 0), slot("ပ", 14), slot("သစ်", 20)];
        assert_eq!(reuse(&old, &added, (8, 12)), None);
    }
}
