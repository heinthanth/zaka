use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::parse_text;

/// Run a check on a thread with room for deep recursion: frames of a debug
/// build are larger than those the limit is set for.
fn with_stack(check: fn())
{
    let thread = std::thread::Builder::new().stack_size(64 << 20).spawn(check);
    thread.unwrap().join().unwrap();
}

/// An unclosed `(` after two words gets a diagnostic: the scan for the
/// close stops at the end of the input.
#[test]
fn an_unclosed_paren_ends_the_scan()
{
    assert!(!codes("a b(").is_empty());
    assert!(!codes("က ခ (").is_empty());
    assert!(!codes("a b(c။").is_empty());
}

/// Nested parentheses in a declaration with no kind parse at once: a group
/// that does not end in a function kind word is not scanned as one.
#[test]
fn nested_parens_in_a_declaration_parse()
{
    let deep = format!("ကိန်းရှင် ဖ သည် {}၁{} ဖြစ်ပါစေ။", "(".repeat(40), ")".repeat(40));
    assert_eq!(codes(&deep), Vec::<Code>::new());
}

/// Parentheses nested past the limit get ZK0244, not a stack overflow.
#[test]
fn nesting_past_the_limit_is_an_error()
{
    with_stack(|| {
        let deep = format!("{}၁{} ကို ပြသမည်။", "(".repeat(300), ")".repeat(300));
        assert!(codes(&deep).contains(&Code::NestingTooDeep));
        let fine = format!("{}၁{} ကို ပြသမည်။", "(".repeat(100), ")".repeat(100));
        assert!(!codes(&fine).contains(&Code::NestingTooDeep));
    });
}

/// An operator after a value read with a verb gets the hint to put the
/// clause in parentheses.
#[test]
fn an_operator_after_a_verb_hints_at_parentheses()
{
    let out = parse_text("အကယ်၍ xs ၏ အကွက်အရေအတွက် == ၀ ဖြစ်လျှင် ပြန်ပေးမည်။");
    let help = out.diagnostics[0].help.clone().unwrap_or_default();
    assert!(help.contains("parentheses"), "{help}");
}
