use zakac_explorer::compile::Options;
use zakac_explorer::compile::compile;
use zakac_explorer::compile::samples::SAMPLES;
use zakac_explorer::compile::sexpr::Ink;
use zakac_explorer::compile::sexpr::SEXPR_WIDTH;
use zakac_explorer::compile::sexpr::arrange;

/// The laid-out text.
fn text(flat: &str, width: usize) -> String
{
    arrange(flat, width).into_iter().map(|c| c.text).collect()
}

/// The text with all whitespace removed; arranging adds nothing but
/// whitespace, so this must not change.
fn squeeze(text: &str) -> String { text.split_whitespace().collect() }

#[test]
fn short_forms_stand_on_one_line()
{
    assert_eq!(text("(var x i32 1)", 60), "(var x i32 1)");
    assert_eq!(text("(block (print 1))", 60), "(block (print 1))");
    assert_eq!(text("(block)", 60), "(block)");
}

#[test]
fn blocks_put_each_statement_on_its_own_line()
{
    let flat = "(block (var a i32 1) (var b i32 2))";
    assert_eq!(text(flat, 60), "(block\n  (var a i32 1)\n  (var b i32 2))");
    let flat = "(program (var a i32 1) (var b i32 2))";
    assert_eq!(text(flat, 60), "(program\n  (var a i32 1)\n  (var b i32 2))");
}

#[test]
fn long_lists_line_up_under_the_first_argument()
{
    let flat =
        "(program (if (> alpha beta) (block (print alpha)) else (block (print beta))))";
    let want = "(program\n  (if (> alpha beta)\n      (block (print alpha))\n      \
                else\n      (block (print beta))))";
    assert_eq!(text(flat, 30), want);
}

#[test]
fn nested_blocks_break_their_parents()
{
    let flat = "(while c (block (set a 1) (set b 2)))";
    let want = "(while c\n       (block\n         (set a 1)\n         (set b 2)))";
    assert_eq!(text(flat, 60), want);
}

#[test]
fn quotes_keep_brackets_and_spaces()
{
    let flat = "(block (print \"a ) b\") (print ')') (print ' '))";
    let out = text(flat, 60);
    assert_eq!(out, "(block\n  (print \"a ) b\")\n  (print ')')\n  (print ' '))");
    let quotes: Vec<String> = arrange(flat, 60)
        .into_iter()
        .filter(|c| c.ink == Ink::Quote)
        .map(|c| c.text)
        .collect();
    assert_eq!(quotes, ["\"a ) b\"", "')'", "' '"]);
}

#[test]
fn heads_are_marked()
{
    let heads: Vec<String> = arrange("(+ (- a b) c)", 60)
        .into_iter()
        .filter(|c| c.ink == Ink::Head)
        .map(|c| c.text)
        .collect();
    assert_eq!(heads, ["+", "-"]);
}

#[test]
fn arrange_adds_nothing_but_whitespace()
{
    for sample in SAMPLES
    {
        let flat = compile(sample.text, &Options::default()).sexpr;
        let out = text(&flat, SEXPR_WIDTH);
        assert_eq!(squeeze(&out), squeeze(&flat), "{}", sample.name);
    }
}

#[test]
fn unmatched_brackets_are_kept()
{
    assert_eq!(squeeze(&text(") (a", 60)), ")(a)");
}
