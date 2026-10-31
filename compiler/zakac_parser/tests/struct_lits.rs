mod common;

use common::codes;
use common::stmts;
use zakac_ast::expr::Expr;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Code;

const DECL: &str = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက်၊ ကိန်းပြည့် အတန်း တို့ ပါဝင်သည်။\n";

/// The tree of the print statement that shows `value`, after [`DECL`].
fn shown(value: &str) -> String
{
    let list = stmts(&format!("{DECL}{value} ကို ပြသပါ။"));
    list.last().expect("a print statement").to_sexpr()
}

/// The struct name and classifier of the literal a print statement shows.
fn count_of(value: &str) -> (String, String)
{
    let list = stmts(&format!("{DECL}{value} ကို ပြသပါ။"));
    let Some(Stmt::Print(print)) = list.last()
    else
    {
        panic!("{value}: no print statement");
    };
    let zakac_ast::stmt::PrintArg::Expr(Expr::Struct(lit)) = &print.value
    else
    {
        panic!("{value}: no struct literal");
    };
    (lit.name.name.clone(), lit.classifier.name.clone())
}

#[test]
fn a_struct_literal_pairs_each_field_with_its_value()
{
    assert_eq!(
        shown("အသက် ၂၀၊ အတန်း ၁ + ၂ ရှိသော လူတစ်ခု"),
        "(print (struct လူ (အသက် 20) (အတန်း (+ 1 2))))"
    );
    assert_eq!(shown("အတန်း -၃ ရှိသော လူတစ်ခု"), "(print (struct လူ (အတန်း (neg 3))))");
}

#[test]
fn the_count_is_one_word_or_two()
{
    let joined = ("လူ".to_string(), "ယောက်".to_string());
    assert_eq!(count_of("အသက် ၂၀ ရှိသော လူတစ်ယောက်"), joined);
    assert_eq!(count_of("အသက် ၂၀ ရှိသော လူ တစ်ယောက်"), joined);
}

#[test]
fn an_undeclared_name_splits_at_one_for_the_checker()
{
    let unknown = ("ကျား".to_string(), "ကောင်".to_string());
    assert_eq!(count_of("အသက် ၂၀ ရှိသော ကျားတစ်ကောင်"), unknown);
    assert_eq!(count_of("အသက် ၂၀ ရှိသော ကျား တစ်ကောင်"), unknown);
}

#[test]
fn a_literal_inside_a_literal_takes_parentheses()
{
    let text = "ဒေတာအမျိုးအစား မျဉ်း တွင် လူ စ၊ လူ ဆုံး တို့ ပါဝင်သည်။\n";
    let program =
        format!("{text}စ (အသက် ၁ ရှိသော လူတစ်ခု)၊ ဆုံး (အတန်း ၂ ရှိသော လူတစ်ခု) ရှိသော မျဉ်းတစ်ခု");
    assert_eq!(
        shown(&program),
        "(print (struct မျဉ်း (စ [(struct လူ (အသက် 1))]) (ဆုံး [(struct လူ (အတန်း 2))])))"
    );
}

#[test]
fn a_literal_needs_its_count()
{
    assert_eq!(codes(&format!("{DECL}အသက် ၂၀ ရှိသော လူ ကို ပြသပါ။")), [Code::ExpectedToken]);
    let no_word = format!("{DECL}အသက် ၂၀ ရှိသော လူတစ် ကို ပြသပါ။");
    assert_eq!(codes(&no_word), [Code::ExpectedToken]);
}
