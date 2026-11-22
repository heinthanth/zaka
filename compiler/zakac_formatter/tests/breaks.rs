//! The lines of a declaration: a field or a member on each line, and the
//! name, the parameters and the opener of a function signature on lines of
//! their own.

use zakac_formatter::FmtOptions;
use zakac_formatter::options::Lines;

use crate::common::fmt;
use crate::common::fmt_with;
use crate::common::repo;

/// One item of a file: the lines from the line that starts with a word to
/// the next blank line, each with its line break.
///
/// # Arguments
///
/// * `path` - The path inside the repo.
/// * `start` - The first word of the first line.
///
/// # Returns
///
/// The lines.
fn item(path: &str, start: &str) -> String
{
    let text = std::fs::read_to_string(repo().join(path)).unwrap();
    let lines = text.lines().skip_while(|l| !l.starts_with(start));
    lines.take_while(|l| !l.is_empty()).map(|l| format!("{l}\n")).collect()
}

/// The settings with a narrow line.
///
/// # Arguments
///
/// * `width` - The widest line.
///
/// # Returns
///
/// The settings.
fn narrow(width: usize) -> FmtOptions { FmtOptions::builder().max_width(width).build() }

/// A struct past the widest line puts its head, each field and its end on
/// lines of their own, each separator at the end of its line.
#[test]
fn a_long_struct_puts_each_field_on_a_line()
{
    let text = item("samples/basics/enums.zaka", "ခရီး ");
    let text = fmt_with(&text, &narrow(1000));
    assert_eq!(text.lines().count(), 1, "a wide line puts it on one line: {text}");
    let out = fmt_with(&text, &narrow(40));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 5, "{out}");
    assert!(lines[0].ends_with("ဒေတာအစုအဝေးတွင်"), "{out}");
    assert!(lines[1].ends_with('၊'), "{out}");
    assert!(lines[2].ends_with("နှင့်"), "{out}");
    assert!(lines[4].starts_with("တို့ပါဝင်"), "{out}");
    assert_eq!(fmt_with(&out, &narrow(1000)), text, "a wide line joins it back");
}

/// The mark of the initial member remains on the line of its member.
#[test]
fn the_initial_mark_keeps_to_its_member()
{
    let text = item("samples/basics/enums.zaka", "အရောင် ");
    let out = fmt_with(&text, &narrow(30));
    let second = out.lines().nth(1).unwrap();
    assert!(second.starts_with("ကနဦးတန်ဖိုး"), "{out}");
}

/// A signature past the widest line puts its name, each parameter and its
/// opener on lines of their own; `လက်ခံပြီး` follows the last parameter.
#[test]
fn a_long_signature_puts_each_parameter_on_a_line()
{
    let text = item("stdlib/text.zaka", "ကိုက်ညီမှုရှိမရှိစစ်");
    let options = FmtOptions::builder().signature_lines(Lines::Split).build();
    let out = fmt_with(&text, &options);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].ends_with("သည်"), "{out}");
    assert!(lines[3].ends_with("လက်ခံပြီး"), "{out}");
    assert!(lines[4].starts_with("အောက်ပါတို့ကို"), "{out}");
}

/// The function signature of the sample, written over three lines, and the
/// block after it.
///
/// # Returns
///
/// The text.
fn split_signature() -> String
{
    let text = item("samples/programs/tic_tac_toe.zaka", "ကစားကွက်ပြ ");
    fmt_with(&text, &narrow(1000))
        .replacen(" ကစားကွက် ဟူသော", "\nကစားကွက် ဟူသော", 1)
        .replacen(" အောက်ပါတို့ကို", "\nအောက်ပါတို့ကို", 1)
}

/// A short signature written over several lines goes on one line.
#[test]
fn a_short_signature_goes_on_one_line()
{
    let text = split_signature();
    let out = fmt_with(&text, &narrow(200));
    assert!(out.lines().next().unwrap().ends_with("လုပ်ဆောင်မည်။"), "{out}");
    assert_eq!(out.lines().count(), text.lines().count() - 2, "{out}");
}

/// With `preserve`, a declaration keeps its lines.
#[test]
fn preserve_keeps_the_lines()
{
    let text = split_signature();
    let options = FmtOptions::builder().signature_lines(Lines::Preserve).build();
    assert_eq!(fmt_with(&text, &options), text);
}

/// A declaration with a comment inside keeps its lines, as a comment ends
/// its line.
#[test]
fn a_comment_inside_keeps_the_lines()
{
    let text =
        split_signature().replacen("တစ်ခု ကို လက်ခံပြီး", "တစ်ခု ကို လက်ခံပြီး // the board", 1);
    assert_eq!(fmt_with(&text, &narrow(200)), text);
}

/// The stdlib module of text, formatted.
///
/// # Returns
///
/// The text.
fn text_module() -> String
{
    std::fs::read_to_string(repo().join("stdlib/text.zaka")).unwrap()
}

/// A line of the opener still too wide breaks after the opener, and a result
/// kind still too wide breaks before its `သို့မဟုတ်`.
#[test]
fn a_wide_result_breaks_after_the_opener()
{
    let out = fmt(&text_module());
    assert!(out.contains("\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ\n"), "{out}");
    assert!(out.contains("\nသို့မဟုတ် အမှားအယွင်းအနေဖြင့် စာလုံးဖတ်ခြင်းအမှားအယွင်း"), "{out}");
}

/// A block written as one statement goes on a line of its own when its line
/// is too wide, and remains after its condition when it fits.
#[test]
fn a_wide_statement_goes_on_a_line_of_its_own()
{
    let out = fmt(&text_module());
    assert!(out.contains("အကယ်၍ အရေအတွက် == ၀ လျှင်\nစာလုံးဖတ်ခြင်း"), "{out}");
    assert!(out.contains("လျှင် မမှန် ကို ပြန်ပေးမည်။"), "{out}");
    let split = out.replacen("လျှင် မမှန် ကို ပြန်ပေးမည်။", "လျှင်\nမမှန် ကို ပြန်ပေးမည်။", 1);
    assert_eq!(fmt(&split), out, "a short statement joins its condition");
}
