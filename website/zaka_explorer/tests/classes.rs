use zaka_explorer::compile::Options;
use zaka_explorer::compile::compile;

/// Editor classes of the tokens of `source`, in order.
///
/// # Arguments
///
/// * `source` - The source text.
///
/// # Returns
///
/// The classes.
fn classes(source: &str) -> Vec<&'static str>
{
    let out = compile(source, &Options::default());
    out.tokens.iter().map(|t| t.class).collect()
}

#[test]
fn print_statement_splits_into_roles()
{
    let want = ["tok-ident", "tok-particle", "tok-verb", "tok-delim"];
    assert_eq!(classes("က ကို ပြသပါ။"), want);
}

#[test]
fn operators_stand_apart_from_brackets()
{
    let want = [
        "tok-delim",
        "tok-number",
        "tok-op",
        "tok-number",
        "tok-delim",
        "tok-particle",
        "tok-verb",
        "tok-delim",
    ];
    assert_eq!(classes("(၁ + ၂) ကို ပြသပါ။"), want);
}

#[test]
fn word_operators_kinds_and_truth_values_get_their_own_classes()
{
    let found = classes("က သည် ကိန်းပြည့် ဖြစ်ပါစေ။ ခ ကို မှန် ဟု ထားပါ။");
    assert!(found.contains(&"tok-kind"));
    assert!(found.contains(&"tok-bool"));
    let sum = classes("က နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသပါ။");
    assert!(sum.contains(&"tok-op"));
}
