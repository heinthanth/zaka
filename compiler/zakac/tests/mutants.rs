//! A check for crashes alone: the programs of the samples and of the fences of
//! the books, each changed a little, go through each phase, a short run on the
//! VM and the C target. Each must end
//! in diagnostics, output or a stop, never in a panic or a hang. The changes
//! come from a fixed seed, so a failure repeats; `ZAKA_MUTANT_SEED` and
//! `ZAKA_MUTANT_COUNT` pick others, and `ZAKA_MUTANT_LAST` keeps the last one.
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen_target_c::driver::translate;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_codegen_target_vm::artifact::run_phases;
use zakac_codegen_target_vm::options::EmitOptions;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The ops a mutant runs on the VM at most.
const OPS: u64 = 200_000;

/// The time one mutant takes at most before it counts as a hang.
const LIMIT: Duration = Duration::from_secs(20);

/// The pieces a mutant can gain: brackets, quotes, the sentence and list
/// marks, a digit at an edge, a zero-width joiner and a closer word.
const PIECES: &[&str] = &[
    "(",
    ")",
    "[",
    "]",
    "\"",
    "{",
    "}",
    "။",
    "၊",
    "၀",
    "၉၉၉၉၉၉၉၉၉၉၉၉၉၉၉၉၉၉၉၉",
    "-",
    "\u{200d}",
    "ပြီးဆုံးပါပြီး",
    "ထဲမှ",
    "ကို",
];

/// A small generator of numbers from a seed: the same seed gives the same
/// numbers on each host.
struct Rng(u64);

impl Rng
{
    /// The next number below `n`, which must be above 0.
    fn below(&mut self, n: usize) -> usize
    {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// A number from the environment, or the default.
fn env_number(name: &str, default: u64) -> u64
{
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// The repo folder.
fn repo() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..") }

/// The files under a folder with an extension, in a fixed order.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `ext` - The extension, with no dot.
///
/// # Returns
///
/// The paths.
fn files_under(dir: &Path, ext: &str) -> Vec<PathBuf>
{
    let entries = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path());
    let mut found: Vec<PathBuf> = entries
        .flat_map(|p| match p.is_dir()
        {
            true => files_under(&p, ext),
            false => vec![p]
                .into_iter()
                .filter(|p| p.extension().is_some_and(|e| e == ext))
                .collect(),
        })
        .collect();
    found.sort();
    found
}

/// The Zaka fences of a page: each block that opens with a line that starts
/// with three backticks and `zaka`, up to the line that closes it. A
/// `zaka-file` fence is one file of a program in several, so it is left out.
///
/// # Arguments
///
/// * `page` - The Markdown text.
///
/// # Returns
///
/// The programs.
fn fences(page: &str) -> Vec<String>
{
    let mut found = Vec::new();
    let mut open: Option<Vec<&str>> = None;
    for line in page.lines()
    {
        let fence = line.starts_with("```");
        let starts = line.starts_with("```zaka") && !line.starts_with("```zaka-file");
        match (open.take(), fence)
        {
            (Some(lines), true) => found.push(lines.join("\n")),
            (Some(mut lines), false) =>
            {
                lines.push(line);
                open = Some(lines);
            }
            (None, true) if starts => open = Some(Vec::new()),
            (None, _) => (),
        }
    }
    found
}

/// The program of each sample and of each Zaka fence of each book.
fn programs() -> Vec<String>
{
    let read = |p: PathBuf| std::fs::read_to_string(p).unwrap();
    let samples = files_under(&repo().join("samples"), "zaka").into_iter().map(read);
    let pages = files_under(&repo().join("docs"), "md").into_iter().map(read);
    samples.chain(pages.flat_map(|p| fences(&p))).collect()
}

/// A program changed once: a line cut, doubled or moved, a word or a
/// character cut, a piece put in, or the end cut off.
///
/// # Arguments
///
/// * `text` - The program.
/// * `rng` - The generator.
///
/// # Returns
///
/// The changed program.
fn mutate(text: &str, rng: &mut Rng) -> String
{
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let chars: Vec<char> = text.chars().collect();
    let line = rng.below(lines.len().max(1));
    let other = rng.below(lines.len().max(1));
    let at = rng.below(chars.len().max(1));
    match rng.below(7)
    {
        0 if !lines.is_empty() => _ = lines.remove(line),
        1 if !lines.is_empty() => lines.insert(line, lines[line].clone()),
        2 if lines.len() > 1 => lines.swap(line, other),
        3 => return cut_word(&lines, line, rng),
        4 =>
        {
            return chars
                .iter()
                .enumerate()
                .filter(|(n, _)| *n != at)
                .map(|(_, c)| c)
                .collect();
        }
        5 =>
        {
            let piece = PIECES[rng.below(PIECES.len())];
            let (head, tail) = chars.split_at(at);
            return format!(
                "{}{piece}{}",
                String::from_iter(head),
                String::from_iter(tail)
            );
        }
        _ => return chars[..at].iter().collect(),
    }
    lines.join("\n")
}

/// The program with one word of one line cut.
fn cut_word(lines: &[String], line: usize, rng: &mut Rng) -> String
{
    let mut lines = lines.to_vec();
    let Some(text) = lines.get(line)
    else
    {
        return String::new();
    };
    let mut words: Vec<&str> = text.split(' ').collect();
    words.remove(rng.below(words.len()));
    lines[line] = words.join(" ");
    lines.join("\n")
}

/// The words of a C function. A C function trusts what it is given, so a
/// change to its arguments, such as a format that reads a number as text,
/// can crash inside C as it would in a C program: such a program goes
/// through each phase but does not run.
const FOREIGN: &str = "မှလာသော";

/// Run one program through each phase, a short run on the VM, and the C
/// target as far as its C text.
fn each_phase(text: String)
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("main.zaka".into()), text);
    let lexer = LexerOptions::default();
    let semcheck = SemcheckOptions::default();
    let mem = BorrowCheckOptions::default();
    let fold = FoldOptions::default();
    let emitted =
        run_phases(&file, &lexer, &semcheck, &fold, &mem, EmitOptions::default());
    let runs = !file.src.contains(FOREIGN);
    if let Some(module) = emitted.module.filter(|_| runs)
    {
        let options = RunOptions::builder().max_steps(OPS).build();
        let _ = run_and_keep(
            &module,
            &mut Vec::new(),
            &options,
            &HostData::program("main.zaka"),
        );
    }
    let loaded = Loaded::single(&file, &lexer);
    let _ = translate(
        loaded,
        sm.files(),
        &semcheck,
        &fold,
        &mem,
        TargetCOptions::default(),
    );
}

/// Whether one program goes through each phase in time with no panic.
///
/// # Returns
///
/// `None` when it does, else what went wrong.
fn failure(text: &str) -> Option<String>
{
    // A crash ends the whole test, so `ZAKA_MUTANT_LAST` names a file that
    // keeps the mutant that runs, to find the one that crashed.
    if let Ok(path) = std::env::var("ZAKA_MUTANT_LAST")
    {
        std::fs::write(path, text).unwrap();
    }
    let (send, receive) = mpsc::channel();
    let owned = text.to_string();
    let thread = std::thread::Builder::new().stack_size(256 << 20).spawn(move || {
        let ran = std::panic::catch_unwind(|| each_phase(owned));
        let _ = send.send(ran.is_ok());
    });
    thread.unwrap();
    match receive.recv_timeout(LIMIT)
    {
        Ok(true) => None,
        Ok(false) => Some(format!("panicked on\n{text}")),
        Err(_) => Some(format!("ran past {LIMIT:?} on\n{text}")),
    }
}

#[test]
fn programs_changed_a_little_never_panic_or_hang()
{
    let programs = programs();
    assert!(
        programs.len() > 500,
        "the samples and books hold programs: {}",
        programs.len()
    );
    let mut rng = Rng(env_number("ZAKA_MUTANT_SEED", 0x5a4b_4121).max(1));
    let count = env_number("ZAKA_MUTANT_COUNT", 1000);
    let mutants = (0..count).map(|_| {
        let program = &programs[rng.below(programs.len())];
        mutate(program, &mut rng)
    });
    let found: Vec<String> = mutants.filter_map(|m| failure(&m)).collect();
    assert!(
        found.is_empty(),
        "{} mutants failed:\n\n{}",
        found.len(),
        found.join("\n---\n")
    );
}
