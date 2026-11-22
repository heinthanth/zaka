//! The wrap of a line too wide: at the end of a clause first, then after a
//! `၊`, then at a space after a reserved word.

use zakac_formatter::FmtOptions;

use crate::common::fmt;
use crate::common::fmt_with;
use crate::common::repo;

/// The text of a file of the repo.
///
/// # Arguments
///
/// * `path` - The path inside the repo.
///
/// # Returns
///
/// The text.
fn read(path: &str) -> String { std::fs::read_to_string(repo().join(path)).unwrap() }

/// The text of a file with one statement written on one line: the lines
/// from the line that starts with a word to the line that ends in `။`.
///
/// # Arguments
///
/// * `path` - The path inside the repo.
/// * `start` - The first word of the statement.
///
/// # Returns
///
/// The text.
fn one_line(path: &str, start: &str) -> String
{
    let text = read(path);
    let lines: Vec<&str> = text.lines().collect();
    let first = lines.iter().position(|l| l.starts_with(start)).unwrap();
    let last = first + lines[first..].iter().position(|l| l.ends_with('။')).unwrap();
    let joined = lines[first..=last].join(" ");
    let mut all: Vec<&str> = lines[..first].to_vec();
    all.push(&joined);
    all.extend(&lines[last + 1..]);
    all.join("\n") + "\n"
}

/// Each line of a text, as its count of characters.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The widths.
fn widths(text: &str) -> Vec<usize>
{
    text.lines().map(|l| l.chars().count()).collect()
}

/// A long import list fills its lines, each ending in a `၊`.
#[test]
fn an_import_list_fills_its_lines()
{
    let text = one_line("samples/stdlib/path.zaka", "\"စံကား/");
    assert!(widths(&text)[1] > 100, "{text}");
    let out = fmt(&text);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[1].ends_with('၊') && lines[2].ends_with('၊'), "{out}");
    assert!(widths(&out)[1..4].iter().all(|w| *w <= 100), "{out}");
}

/// A declaration with a value breaks at the end of a clause, so the value
/// starts its own line.
#[test]
fn a_declaration_breaks_before_its_value()
{
    let out = fmt(&one_line("samples/lists/borrow.zaka", "ကိန်းရှင် ဂရစ်"));
    assert!(out.contains("ဖြစ်၍ တန်ဖိုးသည်\n"), "{out}");
}

/// A line with no good place to break, such as one long string and its verb,
/// keeps its words together.
#[test]
fn a_line_with_no_good_break_keeps_its_words()
{
    let options = FmtOptions::builder().max_width(40).build();
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n\"ဤ စာကြောင်း သည် ရှည်လျားသော စာကြောင်း \
                တစ်ကြောင်း ဖြစ်သည်\" ဟု ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let out = fmt_with(text, &options);
    assert!(
        out.lines().any(|l| l.ends_with("ဟု ပြသမည်။") && l.starts_with('"')),
        "{out}"
    );
}

/// With `wrap-long-lines` off, a long line keeps its length.
#[test]
fn the_wrap_can_be_turned_off()
{
    let text = one_line("samples/stdlib/path.zaka", "\"စံကား/");
    let options = FmtOptions::builder().wrap_long_lines(false).build();
    assert_eq!(fmt_with(&text, &options), text);
}
