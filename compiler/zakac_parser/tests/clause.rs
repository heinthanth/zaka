use zakac_diagnostics::Code;

use crate::common;
use crate::common::codes;
use crate::common::expr;
use crate::common::expr_err;
use crate::common::stmt;

#[test]
fn nested_frames_reduce_on_the_inner_verb_first()
{
    assert_eq!(expr("က နှင့် ခ နှင့် ဂ ကို မြှောက်သည့်ရလဒ် ကို ပေါင်းသည့်ရလဒ်"), "(add က (mul ခ ဂ))");
}

#[test]
fn a_verb_result_takes_the_next_particle()
{
    assert_eq!(expr("က နှင့် ခ ကို ပေါင်းသည့်ရလဒ် နှင့် ဂ ကို မြှောက်သည့်ရလဒ်"), "(mul (add က ခ) ဂ)");
}

#[test]
fn operands_are_maximal_notation_expressions()
{
    assert_eq!(expr("က + ၁ သည် ခ နှင့် ဂ ကို ပေါင်းသည့်ရလဒ် ထက် ကြီးသည်"), "(gt (+ က 1) (add ခ ဂ))");
    assert_eq!(expr("က * ခ နှင့် ဂ ကို ပေါင်းသည့်ရလဒ်"), "(add (* က ခ) ဂ)");
}

#[test]
fn divide_and_remainder_frames()
{
    let text = "၁၆ ကို ၄ ဖြင့် စားသည့်ရလဒ် နှင့် ၁၀ ကို ၃ ဖြင့် အကြွင်းရှာသည့်ရလဒ် ကို ပေါင်းသည့်ရလဒ်";
    assert_eq!(expr(text), "(add (div 16 4) (rem 10 3))");
}

#[test]
fn shift_distance_is_not_read_as_a_kind()
{
    assert_eq!(expr("၄ ကို ၂ ဖြင့် ဘစ်ညာရွေ့သည့်ရလဒ်"), "(shr 4 2)");
    assert_eq!(expr("၁ ကို က ဖြင့် ဘစ်ဘယ်ရွေ့သည့်ရလဒ်"), "(shl 1 က)");
}

#[test]
fn bitwise_frames()
{
    assert_eq!(expr("က နှင့် ခ ကို ဘစ်ဆုံသည့်ရလဒ်"), "(bitand က ခ)");
    assert_eq!(expr("က နှင့် ခ ကို ဘစ်ပေါင်းစုသည့်ရလဒ်"), "(bitor က ခ)");
    assert_eq!(expr("က နှင့် ခ ကို ဘစ်ခြားနားသည့်ရလဒ်"), "(bitxor က ခ)");
    assert_eq!(expr("က ကို ဘစ်ပြောင်းပြန်သည့်ရလဒ်"), "(bitnot က)");
}

#[test]
fn comparison_verbs()
{
    assert_eq!(expr("က သည် ခ နှင့် ညီသည်"), "(eq က ခ)");
    assert_eq!(expr("က သည် ခ နှင့် မညီပါ"), "(ne က ခ)");
    assert_eq!(expr("က သည် ခ ထက် မကြီး"), "(le က ခ)");
    assert_eq!(expr("က သည် ခ အောက် ငယ်"), "(lt က ခ)");
    assert_eq!(expr("က သည် ခ အောက် မငယ်သည်"), "(ge က ခ)");
    assert_eq!(expr("က သည် ခ ထက်ကြီးသည်"), "(gt က ခ)");
}

#[test]
fn conversion_takes_a_kind_operand()
{
    assert_eq!(expr("၁၁.၁၁ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"), "(cast 11.11 isize)");
    assert_eq!(expr("က * ခ ကို ကိန်းပြည့် တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်"), "(cast (* က ခ) isize)");
    let text = "အခြေ ၁၆ ဖြင့် FF ဟူသော လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့် တစ်ခု ကို ဒဿမကိန်း တစ်ခု အဖြစ် \
                ပြောင်းလဲသော ရလဒ် ကို ပြသမည်။";
    assert_eq!(stmt(text), "(print (cast 255u8 f64))");
    let sized = "က ကို ၃၂ ဘစ် ဒဿမကိန်း တစ်ခု အဖြစ် ပြောင်းလဲသော ရလဒ်";
    assert_eq!(expr(sized), "(cast က f32)");
}

#[test]
fn statement_verbs_close_the_clause()
{
    let text = "အသက် ကို အသက် နှင့် ၁ ကို ပေါင်းသည့်ရလဒ် ဟု ထားမည်။";
    assert_eq!(stmt(text), "(set အသက် (add အသက် 1))");
    assert_eq!(stmt("က နှင့် ခ ကို ပေါင်းသည့်ရလဒ် ကို ပြသမည်။"), "(print (add က ခ))");
    assert_eq!(stmt("က + ခ + ဂ ကို ပြန်ပေးမည်။"), "(return (+ (+ က ခ) ဂ))");
    assert_eq!(stmt("ရလဒ် ကို (က + ခ) * ၂ ဟု ထားမည်။"), "(set ရလဒ် (* [(+ က ခ)] 2))");
}

#[test]
fn a_call_can_head_a_statement()
{
    let text = "နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။";
    assert_eq!(stmt(text), "(print (call နှစ်ဆ 1))");
    let sum = "f ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် နှင့် ၁ ကို ပေါင်းသည့်ရလဒ် ကို ပြသမည်။";
    assert_eq!(stmt(sum), "(print (add (call f) 1))");
}

#[test]
fn particle_errors()
{
    assert_eq!(expr_err("က ကို ကို"), Code::DoubleParticle);
    assert_eq!(expr_err("က ကို ပေါင်းသည့်ရလဒ်"), Code::MissingOperand);
    assert_eq!(expr_err("က ကို ခ ကို ပေါင်းသည့်ရလဒ်"), Code::SlotMismatch);
    assert_eq!(expr_err("က နှင့် ခ ကြီး"), Code::SlotMismatch);
    assert_eq!(expr_err("က နှင့် ခ"), Code::UnclosedClause);
    assert_eq!(expr_err("က ကို ပြသမည်"), Code::StmtVerbInExpr);
}

#[test]
fn statement_level_clause_errors()
{
    assert_eq!(codes("ဂ နှင့် က ကို ပြသမည်။"), [Code::DanglingOperands]);
    assert_eq!(codes("(က ကို ပြသမည်) ကို ပြသမည်။"), [Code::StmtVerbInExpr]);
    assert_eq!(stmt("၁ ကို ၂ ဟု ထားမည်။"), "(set 1 2)");
    assert_eq!(codes("က + ၁ ။"), [Code::ExpectedStmtVerb]);
}

#[test]
fn comparisons_do_not_chain()
{
    assert_eq!(expr_err("က သည် ခ နှင့် ညီ သည် ဂ နှင့် ညီသည်"), Code::ChainedComparison);
    assert_eq!(expr_err("က < ခ < ဂ"), Code::ChainedComparison);
    assert_eq!(expr_err("က < ခ သည် ဂ နှင့် ညီ"), Code::ChainedComparison);
    assert_eq!(expr("(က < ခ) သည် ဂ နှင့် ညီ"), "(eq [(< က ခ)] ဂ)");
}

#[test]
fn a_name_joined_to_a_particle_gets_a_hint()
{
    let out = common::parse_text("အသက်ကို ပြသမည်။");
    assert_eq!(out.diagnostics.len(), 1);
    let help = out.diagnostics[0].help.clone().unwrap_or_default();
    assert!(help.contains("အသက် ကို"), "{help}");
}
