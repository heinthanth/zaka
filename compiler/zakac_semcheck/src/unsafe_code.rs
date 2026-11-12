//! Move the pointer work of a program into unsafe code, for tools and tests
//! that hold programs written before pointers needed unsafe code. Statements at
//! the top level move into a start block first, as `in_start_block` moves
//! them, with each variable no function uses. Then each top-level
//! function that works with pointers becomes an unsafe function, and the
//! statements of a start block that does go into an unsafe
//! block. The spellings that left the language go too:
//! `ငှားထားသော`, `လိပ်စာအကြမ်း` and `၏ တန်ဖိုးအငှား`.

use std::collections::BTreeSet;

use zakac_ast::stmt::DeclStmt;
use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_parser::parser::parse;
use zakac_parser::wrap::Wrapped;
use zakac_parser::wrap::wrap_in_start;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

use crate::semcheck::SemcheckOptions;
use crate::semcheck::semcheck;

/// The most rounds the check runs: each round makes unsafe the functions
/// that the round before made callers of unsafe code.
const ROUNDS: usize = 16;

/// The word of an unsafe function.
const UNSAFE_WORD: &str = "စိတ်မချရသော ";

/// The word that opens a function header.
const FUNCTION: &str = "လုပ်ဆောင်ချက် ";

/// The export mark before a function header.
const EXPORT: &str = "အများသုံး ";

/// The word that opens a start block.
const START: &str = "ပရိုဂရမ်သည်";

/// The closer of a start block.
const START_CLOSE: &str = "ပရိုဂရမ် ပြီးဆုံးပါပြီး။";

/// The opener of an unsafe block.
const UNSAFE_OPEN: &str = "အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။";

/// The closer of a block.
const BLOCK_CLOSE: &str = "လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";

/// Move the pointer work of a program into unsafe code.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The program; as it is when it needs no unsafe code or does not parse.
pub fn in_unsafe_code(text: &str) -> String { in_unsafe_code_lines(text).text }

/// [`in_unsafe_code`], with the line each new line comes from. The rewrite
/// keeps each line in its place, so the lines are those the start block gives.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The new program and its lines.
pub fn in_unsafe_code_lines(text: &str) -> Wrapped
{
    // A variable whose value works with pointers moves into the start block,
    // which a mark can reach, unless a function uses it. Its value can call a
    // function that a later round marks, so the moves repeat until no new
    // variable needs one.
    let mut names = BTreeSet::new();
    for _ in 0..ROUNDS
    {
        let wrapped = wrap(text, &names);
        let text = made_unsafe(wrapped.text);
        let lines: Vec<&str> = text.split('\n').collect();
        let more: BTreeSet<String> = lines_outside_unsafe(&text)
            .into_iter()
            .filter(|&n| owner(&lines, n).is_none())
            .filter_map(|n| var_name(lines[n]))
            .collect();
        ret_if!(more.is_subset(&names), Wrapped { text, lines: wrapped.lines });
        names.extend(more);
    }
    let wrapped = wrap(text, &names);
    Wrapped { text: made_unsafe(wrapped.text), lines: wrapped.lines }
}

/// Make unsafe the functions and the start block of a program, round after
/// round, until each line of pointer work stands in unsafe code or a round
/// changes nothing.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The rewritten program.
fn made_unsafe(mut text: String) -> String
{
    for _ in 0..ROUNDS
    {
        let lines = lines_outside_unsafe(&text);
        do_if!(lines.is_empty(), break);
        let next = make_unsafe(&text, &lines);
        do_if!(next == text, break);
        text = next;
    }
    text
}

/// Put the statements at the top level of a program into a start block,
/// with each variable `moved` names; the program as it is when it has a start
/// block or does not parse.
///
/// # Arguments
///
/// * `text` - The program.
/// * `moved` - The variables that move into the start block, unless a function
///   uses them.
///
/// # Returns
///
/// The program and its lines.
fn wrap(text: &str, moved: &BTreeSet<String>) -> Wrapped
{
    let keep = |d: &DeclStmt, seen: bool, used: bool| {
        used || (!seen && !moved.contains(d.name.name.as_str()))
    };
    wrap_in_start(text, keep).unwrap_or_else(|| {
        let lines = (0..text.split('\n').count()).map(Some).collect();
        Wrapped { text: text.to_string(), lines }
    })
}

/// The variable a line declares: `ကိန်းရှင် <name>` or `ကိန်းသေ <name>`.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The name, or `None` for another line.
fn var_name(line: &str) -> Option<String>
{
    let mut words = line.split_whitespace();
    let head = words.next()?;
    ret_if!(head != "ကိန်းရှင်" && head != "ကိန်းသေ", None);
    words.next().map(str::to_string)
}

/// The lines, from 0, where the check finds pointer work outside unsafe
/// code, or a C function that needs the unsafe word.
///
/// # Arguments
///
/// * `text` - The program.
///
/// # Returns
///
/// The lines; none when the program does not parse.
fn lines_outside_unsafe(text: &str) -> Vec<usize>
{
    let mut sm = SourceMap::new();
    let file =
        sm.new_source_file(FileName::Custom("mark.zaka".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let mut parsed = parse(&file, &lexed);
    ret_if!(
        !lexed.diagnostics.is_empty() || !parsed.diagnostics.is_empty(),
        Vec::new()
    );
    let out = semcheck(&mut parsed.program, &SemcheckOptions::default());
    let base = file.start_pos.as_u32() as usize;
    let line = |lo: usize| {
        text[..lo.saturating_sub(base).min(text.len())].matches('\n').count()
    };
    let marks = [Code::UnsafeOutside, Code::ForeignNeedsUnsafe];
    let found = out.diagnostics.iter().filter(|d| marks.contains(&d.code));
    found.map(|d| line(d.span.lo.as_u32() as usize)).collect()
}

/// Make the top-level function of each line unsafe, or wrap the statements of
/// the start block in an unsafe block.
///
/// # Arguments
///
/// * `text` - The program.
/// * `lines` - The lines with pointer work, from 0.
///
/// # Returns
///
/// The program with the marks.
fn make_unsafe(text: &str, lines: &[usize]) -> String
{
    let mut out: Vec<String> = text.split('\n').map(str::to_string).collect();
    let owners: BTreeSet<usize> =
        lines.iter().filter_map(|&n| owner(&out, n)).collect();
    let (starts, fns): (Vec<usize>, Vec<usize>) =
        owners.into_iter().partition(|&at| out[at].starts_with(START));
    for at in fns
    {
        let head = export_len(&out[at]);
        out[at].insert_str(head, UNSAFE_WORD);
    }
    do_if!(!starts.is_empty(), wrap_start(&mut out));
    out.join("\n")
}

/// The length of the export mark a line starts with, 0 for none.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// The length in octets.
fn export_len(line: &str) -> usize { tern!(line.starts_with(EXPORT), EXPORT.len(), 0) }

/// The line of the top-level function header or the start block a line
/// stands in.
///
/// # Arguments
///
/// * `lines` - The lines.
/// * `n` - The line.
///
/// # Returns
///
/// The line of the header, or `None`.
fn owner<S: AsRef<str>>(lines: &[S], n: usize) -> Option<usize>
{
    (0..=n.min(lines.len().saturating_sub(1))).rev().find(|&i| {
        let line = lines[i].as_ref();
        let rest = &line[export_len(line)..];
        line.starts_with(START) || (rest.starts_with(FUNCTION) && is_header(rest))
    })
}

/// Whether a line that starts with `လုပ်ဆောင်ချက်` declares a function:
/// `လုပ်ဆောင်ချက် <name> သည်`, not a call.
///
/// # Arguments
///
/// * `line` - The line.
///
/// # Returns
///
/// `true` for a header.
fn is_header(line: &str) -> bool
{
    let mut words = line.split_whitespace();
    words.next();
    words.next();
    words.next() == Some("သည်")
}

/// Put the statements of the start block in an unsafe block, once. The
/// opener joins the line of the start block's header and the closer the line
/// of its closer, so each line keeps its number.
///
/// # Arguments
///
/// * `lines` - The lines, changed in place.
fn wrap_start(lines: &mut [String])
{
    let Some(head) = lines.iter().position(|l| l.starts_with(START))
    else
    {
        return;
    };
    ret_if!(lines[head].ends_with(UNSAFE_OPEN));
    let Some(close) = lines.iter().rposition(|l| l.starts_with(START_CLOSE))
    else
    {
        return;
    };
    lines[head] = format!("{} {UNSAFE_OPEN}", lines[head]);
    lines[close] = format!("{BLOCK_CLOSE} {}", lines[close]);
}
