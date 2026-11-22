//! The rules of space between words, line breaks, blank lines, depth
//! and the order of the marks.

use zakac_formatter::FmtError;
use zakac_formatter::FmtOptions;
use zakac_formatter::format_text;
use zakac_formatter::options::ContinuationIndent;

use crate::common::fmt;
use crate::common::fmt_with;
use crate::common::repo;

/// Read a file of the tests folder.
///
/// # Arguments
///
/// * `name` - The path inside `tests/`.
///
/// # Returns
///
/// The text.
fn case(name: &str) -> String
{
    let path = repo().join("compiler/zakac_formatter/tests").join(name);
    std::fs::read_to_string(path).unwrap()
}

/// One item of a sample: the lines from the line that starts with a word to
/// the next blank line, each with its line break.
///
/// # Arguments
///
/// * `sample` - The path inside `samples/`.
/// * `start` - The first word of the first line.
///
/// # Returns
///
/// The lines.
fn sample_item(sample: &str, start: &str) -> String
{
    let text = std::fs::read_to_string(repo().join("samples").join(sample)).unwrap();
    let lines = text.lines().skip_while(|l| !l.starts_with(start));
    lines.take_while(|l| !l.is_empty()).map(|l| format!("{l}\n")).collect()
}

/// Extra spaces, spaces at line ends, blank runs and two statements on one
/// line all come back to the clean sample.
#[test]
fn a_rough_file_comes_back_clean()
{
    assert_eq!(fmt(&case("inputs/hello.zaka")), case("expected/hello.zaka"));
}

/// A clean file is left as it is.
#[test]
fn a_clean_file_is_left_as_it_is()
{
    let clean = case("expected/hello.zaka");
    assert_eq!(fmt(&clean), clean);
}

/// With no blank lines kept, the blank run between two statements goes.
#[test]
fn blank_lines_max_cuts_each_run()
{
    let options = FmtOptions::builder().blank_lines_max(0).build();
    let out = fmt_with(&case("expected/hello.zaka"), &options);
    assert!(!out.contains("\n\n"), "{out}");
}

/// With `one_statement_per_line` off, two statements on one line remain there.
#[test]
fn statements_can_share_a_line()
{
    let options = FmtOptions::builder().one_statement_per_line(false).build();
    let out = fmt_with(&case("inputs/hello.zaka"), &options);
    let shared = out.lines().any(|l| l.matches('။').count() == 2);
    assert!(shared, "{out}");
}

/// Each line inside a block is indented once for each block around it; the
/// closer stands at the depth of its opener, and the later lines of the
/// function signature keep their spaces as written.
#[test]
fn indent_follows_the_blocks()
{
    let options = FmtOptions::builder().indent(4).build();
    let out =
        fmt_with(&sample_item("programs/tic_tac_toe.zaka", "ကစားကွက်ပြ "), &options);
    let lead: Vec<usize> =
        out.lines().map(|l| l.len() - l.trim_start().len()).collect();
    assert_eq!(lead, [0, 0, 0, 4, 4, 4, 8, 8, 8, 4, 4, 0], "{out}");
}

/// A number for `continuation_indent` sets the later lines of a statement to
/// that count past the depth of the statement.
#[test]
fn continuation_lines_take_a_set_indent()
{
    let options = FmtOptions::builder()
        .continuation_indent(ContinuationIndent::Spaces(2))
        .build();
    let out = fmt_with(&sample_item("programs/tic_tac_toe.zaka", "နိုင်မနိုင်စစ် "), &options);
    let lead: Vec<usize> =
        out.lines().map(|l| l.len() - l.trim_start().len()).collect();
    assert_eq!(lead[6..9], [2, 2, 2], "{out}");
    assert_eq!(lead[14..16], [2, 2], "{out}");
}

/// With the default settings, the later lines of a statement keep the spaces
/// the author put in front of them.
#[test]
fn continuation_lines_keep_their_spaces()
{
    let text = sample_item("programs/tic_tac_toe.zaka", "နိုင်မနိုင်စစ် ");
    assert_eq!(fmt(&text), text);
}

/// A file written with the dot below ahead of the asat is written in the
/// canonical order, unless `mark_order` is off.
#[test]
fn mark_order_puts_the_asat_first()
{
    let clean = case("expected/hello.zaka");
    let reversed =
        clean.replacen("\u{1004}\u{103a}\u{1037}", "\u{1004}\u{1037}\u{103a}", 1);
    assert_ne!(reversed, clean);
    assert_eq!(fmt(&reversed), clean);
    let options = FmtOptions::builder().mark_order(false).build();
    assert_eq!(fmt_with(&reversed, &options), reversed);
}

/// A file that does not parse is left as it is, with its errors.
#[test]
fn a_broken_file_is_refused()
{
    let broken: String = case("expected/hello.zaka")
        .lines()
        .skip(1)
        .take(2)
        .map(|l| format!("{l}\n"))
        .collect();
    let refused = format_text(&broken, &FmtOptions::default());
    assert!(matches!(refused, Err(FmtError::Unparsed(d)) if !d.is_empty()));
}
