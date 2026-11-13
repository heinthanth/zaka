use zakac_ast::expr::Expr;
use zakac_ast::expr::NextCtx;
use zakac_diagnostics::Code;
use zakac_lexer::token::DigitScript;
use zakac_lexer::token::Particle;
use zakac_span::pos::Pos;

use crate::common::expr;
use crate::common::expr_err;
use crate::common::parse_expr;

#[test]
fn notation_precedence_follows_c()
{
    assert_eq!(expr("၁ + ၂ * ၃"), "(+ 1 (* 2 3))");
    assert_eq!(expr("၁ - ၂ - ၃"), "(- (- 1 2) 3)");
    assert_eq!(expr("က | ခ ^ ဂ & ဃ"), "(| က (^ ခ (& ဂ ဃ)))");
    assert_eq!(expr("က << ၁ + ၂"), "(<< က (+ 1 2))");
    assert_eq!(expr("က + ၁ == ခ"), "(== (+ က 1) ခ)");
    assert_eq!(expr("(က + ခ) * ဂ"), "(* [(+ က ခ)] ဂ)");
}

#[test]
fn prefix_operators_bind_to_the_atom()
{
    assert_eq!(expr("-က * ခ"), "(* (neg က) ခ)");
    assert_eq!(expr("အနုတ် က * ခ"), "(* (neg က) ခ)");
    assert_eq!(expr("အနုတ် ၅"), "(neg 5)");
    assert_eq!(expr("!က == ခ"), "(== (! က) ခ)");
    assert_eq!(expr("~ ~ က"), "(~ (~ က))");
    assert_eq!(expr("၁၀ နှင့် အနုတ် ၅ ၏ နုတ်လဒ်"), "(sub 10 (neg 5))");
}

#[test]
fn atoms()
{
    assert_eq!(expr("'က'"), "'က'");
    assert_eq!(expr("မှန်"), "true");
    assert_eq!(expr("မမှန်ပါ"), "false");
    assert_eq!(expr("`a b`"), "a b");
    assert_eq!(expr("ရလဒ်"), "ရလဒ်");
    assert_eq!(expr("အမြဲတမ်းသုည ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်"), "(call အမြဲတမ်းသုည)");
    let args = "ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ က နှင့် ခ ၏ ပေါင်းလဒ်၊ ဂ + ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်";
    assert_eq!(expr(args), "(call ပေါင်း 1 (add က ခ) (+ ဂ 1))");
}

/// `f(a၊ b)` is the sign form of a call: the same node as the word form.
#[test]
fn a_call_takes_the_sign_form()
{
    let word = "ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၁၊ က နှင့် ခ ၏ ပေါင်းလဒ်၊ ဂ + ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်";
    assert_eq!(expr("ပေါင်း(၁၊ က နှင့် ခ ၏ ပေါင်းလဒ်၊ ဂ + ၁)"), expr(word));
    assert_eq!(expr("အမြဲတမ်းသုည()"), "(call အမြဲတမ်းသုည)");
    assert_eq!(expr("နှစ်ထပ်(နှစ်ထပ်(၂))"), "(call နှစ်ထပ် (call နှစ်ထပ် 2))");
    assert_eq!(expr("နှစ်ထပ်(၃) + ၁"), "(+ (call နှစ်ထပ် 3) 1)");
    assert_eq!(expr("`a b`(၁)"), "(call a b 1)");
}

#[test]
fn integer_literals()
{
    assert_eq!(expr("၁၀"), "10");
    assert_eq!(expr("16"), "16");
    assert_eq!(expr("၁၀ ဟူသော ကိန်းပြည့် တစ်ခု"), "10isize");
    assert_eq!(expr("၁၈ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု"), "18u8");
    assert_eq!(expr("အခြေ ၂ ဖြင့် ၁၀"), "2");
    assert_eq!(expr("အခြေ ၂ ဖြင့် ၁၁၁၁၁၁၁၀ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု"), "254i8");
    assert_eq!(expr("အခြေ ၁၆ ဖြင့် FF ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု"), "255u8");
    assert_eq!(expr("အခြေ ၁၆ ဖြင့် ff"), "255");
    assert_eq!(expr("အခြေ ၁၆ ဖြင့် 1e5"), "485");
    let Ok(Expr::Int(lit)) = parse_expr("၁၂၃")
    else
    {
        panic!("not an integer")
    };
    assert_eq!(lit.script, DigitScript::Burmese);
    assert_eq!(lit.base, 10);
}

#[test]
fn float_literals()
{
    assert_eq!(expr("၁၂.၃၄"), "12.34");
    assert_eq!(expr("12.34"), "12.34");
    assert_eq!(expr("100e+10"), "1000000000000.0");
    assert_eq!(expr("1.234e16"), "1.234e16");
    assert_eq!(expr("၁၀ ဟူသော ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု"), "10.0");
    assert_eq!(expr("-၁၁.၁၁"), "(neg 11.11)");
}

#[test]
fn literal_errors()
{
    assert_eq!(expr_err("အခြေ ၂ ဖြင့် ၁၂"), Code::InvalidDigit);
    assert_eq!(expr_err("အခြေ ၁ ဖြင့် ၀"), Code::InvalidBase);
    assert_eq!(expr_err("အခြေ ၃၇ ဖြင့် ၀"), Code::InvalidBase);
    assert_eq!(expr_err("၂၅၆ ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု"), Code::LiteralOutOfRange);
    assert_eq!(expr_err("၁၂၉ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု"), Code::LiteralOutOfRange);
    assert_eq!(expr_err("၃.၁၄ ဟူသော ကိန်းပြည့် တစ်ခု"), Code::FloatForIntKind);
    assert_eq!(expr_err("၁ ဟူသော စာလုံး တစ်လုံး"), Code::BadLiteralKind);
    assert_eq!(expr_err("အခြေ ၁၀ ဖြင့် FF"), Code::ExpectedToken);
    assert_eq!(expr_err("12ab"), Code::InvalidDigit);
}

#[test]
fn template_literals_hold_expressions_in_their_template_slots()
{
    assert_eq!(expr("\"text\""), "\"text\"");
    assert_eq!(expr("\"က {က + ၁} ခ\""), "\"က {(+ က 1)} ခ\"");
    assert_eq!(expr("\"{က}{ခ}\""), "\"{က}{ခ}\"");
    assert_eq!(expr("\"\" + \"က\""), "(+ \"\" \"က\")");
    assert_eq!(expr("\"a {\"b {က}\"} c\""), "\"a {\"b {က}\"} c\"");
    assert_eq!(expr_err("\"{}\""), Code::ExpectedOperand);
    assert_eq!(expr_err("\"{က ခ}\""), Code::ExpectedToken);
}

#[test]
fn an_error_in_a_template_slot_points_inside_the_literal()
{
    let text = "\"က {ခ +} ဂ\"";
    let err = parse_expr(text).expect_err(text);
    let lo = text.find('{').unwrap();
    let hi = text.find('}').unwrap();
    let at = (err.span.lo - Pos::ONE) as usize;
    assert!((lo..=hi).contains(&at), "{at} not in {lo}..={hi}");
}

#[test]
fn signed_decimal_literal_allows_the_negated_minimum()
{
    assert_eq!(expr("အနုတ် ၁၂၈ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု"), "(neg 128i8)");
    assert_eq!(
        expr("18446744073709551615 ဟူသော လက္ခဏာမဲ့ ၆၄ ဘစ် ကိန်းပြည့် တစ်ခု"),
        "18446744073709551615u64"
    );
}

#[test]
fn kind_errors()
{
    assert_eq!(expr_err("၁ ဟူသော ၇ ဘစ် ကိန်းပြည့် တစ်ခု"), Code::InvalidBitCount);
    assert_eq!(expr_err("၁ ဟူသော ၈ ဘစ် ဒဿမကိန်း တစ်ခု"), Code::InvalidBitCount);
    assert_eq!(expr_err("၁ ဟူသော လက္ခဏာရှိ ဒဿမကိန်း တစ်ခု"), Code::SignedFloat);
}

#[test]
fn logical_words_join_clauses()
{
    let both = "၁ သည် ၀ ထက် ကြီးသည် ဖြစ်၍ ၁ သည် ၂ အောက် ငယ်သည်";
    assert_eq!(expr(both), "(and (gt 1 0) (lt 1 2))");
    assert_eq!(expr("က သို့မဟုတ် ခ ဖြစ်၍ ဂ"), "(or က (and ခ ဂ))");
    assert_eq!(expr("(က သို့မဟုတ် ခ) ဖြစ်၍ ဂ"), "(and [(or က ခ)] ဂ)");
    assert_eq!(expr("က || ခ && ဂ"), "(or က (and ခ ဂ))");
    let cases = "ဖြစ်ရပ်_၁ သို့မဟုတ် ဖြစ်ရပ်_၂ ဖြစ်၍ ဖြစ်ရပ်_၃";
    assert_eq!(expr(cases), "(or ဖြစ်ရပ်_၁ (and ဖြစ်ရပ်_၂ ဖြစ်ရပ်_၃))");
    assert_eq!(expr("က == ခ မဟုတ်ပါ"), "(not (== က ခ))");
    assert_eq!(expr("က သို့မဟုတ် ခ မဟုတ်"), "(or က (not ခ))");
}

#[test]
fn the_word_after_a_verb_is_recorded()
{
    let Ok(Expr::Logic(or)) = parse_expr("အသက် သည် ၁၈ အောက် မငယ်ပါ သို့မဟုတ် မမှန်")
    else
    {
        panic!("not a logical expression")
    };
    let Expr::Verb(lhs) = *or.lhs
    else
    {
        panic!("not a verb")
    };
    assert_eq!(lhs.next, NextCtx::Or);
    assert_eq!(lhs.particle, Particle::Pa);
    let Expr::Bool(rhs) = *or.rhs
    else
    {
        panic!("not a boolean")
    };
    assert_eq!(rhs.next, NextCtx::Other);
}

#[test]
fn pointer_expressions()
{
    assert_eq!(expr("x ၏ လိပ်စာ"), "(addr x)");
    assert_eq!(expr("p ၏ တန်ဖိုး"), "(deref p)");
    assert_eq!(expr("pp ၏ တန်ဖိုး ၏ တန်ဖိုး"), "(deref (deref pp))");
    assert_eq!(expr("p + i ၏ တန်ဖိုး"), "(deref (+ p i))");
    assert_eq!(expr("*p + i"), "(+ (deref p) i)");
    assert_eq!(expr("&x"), "(addr x)");
    assert_eq!(expr("p * ၂"), "(* p 2)");
    assert_eq!(expr("မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ"), "null");
    assert_eq!(expr("ကိန်းပြည့် တစ်ခု ၏ အရွယ်အစား * ၁၀"), "(* (sizeof isize) 10)");
    assert_eq!(expr("၃၂ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ၏ အရွယ်အစား"), "(sizeof i32*)");
    assert_eq!(
        expr("p ကို ၃၂ ဘစ် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"),
        "(cast p i32*)"
    );
    assert_eq!(expr("၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ၏ အရွယ်အစား"), "(sizeof i32*)");
    assert_eq!(expr("ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု ၏ အရွယ်အစား"), "(sizeof isize**)");
    assert_eq!(expr("p ကို ၃၂ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"), "(cast p i32*)");
}

#[test]
fn a_verbal_join_lists_its_pieces()
{
    assert_eq!(expr("\"a\"၊ က၊ \"b\" တို့ကို ဆက်ထားသည့် စာသား"), "(join \"a\" က \"b\")");
}
