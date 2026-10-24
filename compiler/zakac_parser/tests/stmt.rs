mod common;

use common::codes;
use common::parse_text;
use common::stmt;
use common::stmts;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Stmt;
use zakac_ast::visit::Node;
use zakac_diagnostics::Code;

#[test]
fn print_statements()
{
    assert_eq!(stmt("အသက် ကို ပြသပါ။"), "(print အသက်)");
    assert_eq!(stmt("အသက် + ၁ ကို ပြသပါ။"), "(print (+ အသက် 1))");
    assert_eq!(stmt("\"မင်္ဂလာပါ\" ကို ပြသပါ။"), "(print \"မင်္ဂလာပါ\")");
    let logical = "(အသက် သည် ၁၈ အောက် ငယ်သည် မဟုတ်) ကို ပြသပါ။";
    assert_eq!(stmt(logical), "(print [(not (lt အသက် 18))])");
}

#[test]
fn declarations()
{
    assert_eq!(stmt("ကိန်းရှင် အသက် သည် ကိန်းပြည့် ဖြစ်ပါစေ။"), "(var အသက် isize)");
    assert_eq!(stmt("ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။"), "(var အသက် isize 18)");
    assert_eq!(stmt("ကိန်းသေ ပိုင် သည် ဒဿမကိန်း ၃.၁၄ ဖြစ်ပါစေ။"), "(const ပိုင် f64 3.14)");
    let sized = "ကိန်းရှင် ရေတွက် သည် လက္ခဏာမဲ့ ၃၂ ဘစ် ကိန်းပြည့် ဖြစ်ပါစေ။";
    assert_eq!(stmt(sized), "(var ရေတွက် u32)");
    let twice = "ကိန်းရှင် နှစ်ဆ သည် ကိန်းပြည့် အသက် * ၂ ဖြစ်ပါစေ။";
    assert_eq!(stmt(twice), "(var နှစ်ဆ isize (* အသက် 2))");
    let adult = "ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး (အသက် သည် ၁၈ ထက် ကြီးသည်) ဖြစ်ပါစေ။";
    assert_eq!(stmt(adult), "(var ကြီးသူ bool [(gt အသက် 18)])");
    let kinded = "ကိန်းရှင် က သည် ကိန်းပြည့် ၈ ဘစ် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။";
    assert_eq!(stmt(kinded), "(var က isize 5i8)");
    let clause = "ကိန်းရှင် က သည် ကိန်းပြည့် ခ နှင့် ဂ ၏ ပေါင်းလဒ် ဖြစ်ပါစေ။";
    assert_eq!(stmt(clause), "(var က isize (add ခ ဂ))");
}

#[test]
fn assignments()
{
    assert_eq!(stmt("အသက် ကို ၂၄ ဟု ထားပါ။"), "(set အသက် 24)");
    assert_eq!(
        stmt("ကြီးသူ ကို (အသက် သည် ၁၈ ထက် ကြီးသည်) ဟု ထားပါ။"),
        "(set ကြီးသူ [(gt အသက် 18)])"
    );
    assert_eq!(stmt("ဝင်ခွင့်ပြုမည် ကို မမှန် ဟု ထားပါ။"), "(set ဝင်ခွင့်ပြုမည် false)");
}

#[test]
fn blocks_accept_each_closer_spelling()
{
    for closer in ["ပြီးပါပြီ", "ပြီးပါပြီး", "ပြီးဆုံးပါပြီ", "ပြီးဆုံးပါပြီး"]
    {
        let text = format!("အောက်ပါတို့ကို လုပ်ဆောင်ပါ။\nက ကို ပြသပါ။\nလုပ်ဆောင်စရာများ {closer}။");
        assert_eq!(stmt(&text), "(block (print က))", "{closer}");
    }
}

#[test]
fn conditionals()
{
    let text = "အကယ်၍ အသက် သည် ၁၈ အောက် ငယ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဝင်ခွင့်ပြုမည် ကို မမှန် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ထိုသို့မဟုတ်ဘဲ အသက် သည် ၆၀ ထက် ကြီးလျှင် \"ok\" ကို ပြသပါ။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဝင်ခွင့်ပြုမည် ကို မှန် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(if (lt အသက် 18) (block (set ဝင်ခွင့်ပြုမည် false)) (gt အသက် 60) (block \
                (print \"ok\")) else (block (set ဝင်ခွင့်ပြုမည် true)))";
    assert_eq!(stmt(text), want);
    let then_is = "အကယ်၍ အသက် သည် ၁၈ အောက် မငယ်သည် ဖြစ်လျှင် က ကို ပြသပါ။";
    assert_eq!(stmt(then_is), "(if (ge အသက် 18) (block (print က)))");
    let not = "အကယ်၍ ဝင်ခွင့်ပြုမည် မဟုတ်လျှင် လုပ်ဆောင်ချက် ပရိုဂရမ်ပိတ် ကို လုပ်ဆောင်ပါ။";
    assert_eq!(stmt(not), "(if (not ဝင်ခွင့်ပြုမည်) (block (call ပရိုဂရမ်ပိတ်)))");
}

#[test]
fn loops()
{
    let while_loop = "ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့ ကိန်း ကို ပြသပါ။";
    assert_eq!(stmt(while_loop), "(while (lt ကိန်း 5) (block (print ကိန်း)))");
    let counted = "၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။";
    assert_eq!(stmt(counted), "(count ကိန်း 1 10 count (block (print ကိန်း)))");
    let stepped = "၁၀ မှ ၀ အထိ ၂ စီ ကိန်း အနေဖြင့် လျှော့၍ ကိန်း ကို ပြသပါ။";
    assert_eq!(stmt(stepped), "(count ကိန်း 10 0 step 2 down (block (print ကိန်း)))");
    let generic = "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို \
                   လုပ်ဆောင်ပါ။
ကိန်း ကို ပြသပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want =
        "(for ကိန်း isize 0 (lt ကိန်း 10) (block (print ကိန်း)) then (set ကိန်း (+ ကိန်း 1)))";
    assert_eq!(stmt(generic), want);
}

#[test]
fn break_continue_and_return()
{
    let text = "၁ မှ ၁၀၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ ကိန်း % ၂ သည် ၀ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။
အကယ်၍ ကိန်း သည် ၁၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်ပါ။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(count ကိန်း 1 100 count (block (if (eq (% ကိန်း 2) 0) (block \
                (continue))) (if (gt ကိန်း 10) (block (break))) (return)))";
    assert_eq!(stmt(text), want);
}

#[test]
fn functions_and_calls()
{
    let text = "လုပ်ဆောင်ချက် နှစ်ဆတိုးပြီးပေါင်း သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ၊ ကိန်းပြည့် ဂ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ခ + ဂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(fn နှစ်ဆတိုးပြီးပေါင်း [isize က, isize ခ, isize ဂ] isize (block (return (+ \
                (+ က ခ) ဂ))))";
    assert_eq!(stmt(text), want);
    let bare = "လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"မင်္ဂလာပါ\" ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(stmt(bare), "(fn နှုတ်ဆက် [] - (block (print \"မင်္ဂလာပါ\")))");
    let one =
        "လုပ်ဆောင်ချက် f သည် ကိန်းပြည့် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(one), "(fn f [isize က] - (block))");
    let ret = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဒဿမကိန်း ရလဒ် ကို ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ \
               ပြီးပါပြီ။";
    assert_eq!(stmt(ret), "(fn f [] f64 (block))");
    let and = "လုပ်ဆောင်ချက် f သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ နှင့် စာလုံး ဂ တို့ကို လက်ခံပြီး အောက်ပါတို့ကို \
               လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(and), "(fn f [isize က, isize ခ, char ဂ] bool (block))");
    let two = "လုပ်ဆောင်ချက် f သည် ကိန်းပြည့် က နှင့် ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ \
               လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(two), "(fn f [isize က, isize ခ] - (block))");
    assert_eq!(stmt("လုပ်ဆောင်ချက် ပရိုဂရမ်ပိတ် ကို လုပ်ဆောင်ပါ။"), "(call ပရိုဂရမ်ပိတ်)");
    let args = "လုပ်ဆောင်ချက် ထုတ်ပြ ကို အသက်၊ အမည် တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(stmt(args), "(call ထုတ်ပြ အသက် အမည်)");
}

#[test]
fn statement_spans_cover_the_full_stop()
{
    let text = "က ကို ပြသပါ။ ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်ပါစေ။";
    let list = stmts(text);
    let out = parse_text(text);
    let end = |s: &Stmt| (s.span().hi - out.program.span.lo) as usize;
    assert_eq!(&text[..end(&list[0])], "က ကို ပြသပါ။");
    assert!(text[..end(&list[1])].ends_with("ဖြစ်ပါစေ။"));
}

#[test]
fn statement_errors()
{
    assert_eq!(codes("သို့မဟုတ်ပါက က ကို ပြသပါ။"), [Code::ElseWithoutIf]);
    assert_eq!(codes("ပြီးလျှင် က ကို ပြသပါ။"), [Code::UpdateOutsideLoop]);
    assert_eq!(codes("လုပ်ဆောင်စရာများ ပြီးပါပြီ။"), [Code::UnmatchedBlockClose]);
    let nested_if = "အကယ်၍ က လျှင် အကယ်၍ ခ လျှင် ဂ ကို ပြသပါ။";
    assert_eq!(codes(nested_if)[0], Code::NestedSingleLineIf);
    let const_loop = "ကိန်းသေ က သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး က သည် ၁ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို \
                      လုပ်ဆောင်ပါ။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(const_loop)[0], Code::ConstLoop);
    let late = "ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပြီး က သည် ၁ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ပြီးလျှင် က ကို က + ၁ ဟု ထားပါ။
က ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(late), [Code::UpdateNotLast]);
    let nested_fn = "အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(nested_fn), [Code::NestedFunction]);
    let no_result = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(no_result), [Code::ExpectedToken]);
    assert_eq!(codes("\"x\" ကို က ဟု ထားပါ။"), [Code::StrOutsidePrint]);
    assert_eq!(codes("၁ မှ ၁၀ အထိ ၁ + ၁ အနေဖြင့် ရေတွက်၍ က ကို ပြသပါ။"), [Code::ExpectedToken]);
}

#[test]
fn counted_variable_joined_to_its_particle_gets_a_hint()
{
    let out = parse_text("၁ မှ ၁၀ အထိ ကိန်းအနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။");
    assert_eq!(out.diagnostics.len(), 1);
    let help = out.diagnostics[0].help.clone().unwrap_or_default();
    assert!(help.contains("ကိန်း အနေဖြင့်"), "{help}");
}

#[test]
fn errors_recover_at_the_next_statement()
{
    let text = "က ကို ကို ပြသပါ။
ခ ကို ပြသပါ။
အကယ်၍ + လျှင် အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဂ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
သို့မဟုတ်ပါက ဃ ကို ပြသပါ။
င ကို ပြသပါ။";
    let out = parse_text(text);
    let found: Vec<Code> = out.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, [Code::DoubleParticle, Code::ExpectedOperand]);
    let shown: Vec<String> = out.program.stmts.iter().map(ToSexpr::to_sexpr).collect();
    assert_eq!(shown, ["(print ခ)", "(print င)"]);
}

#[test]
fn block_errors_do_not_leak()
{
    let text = "အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
က ကို ကို ပြသပါ။
ခ ကို ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ဂ ကို ပြသပါ။";
    let out = parse_text(text);
    assert_eq!(out.diagnostics.len(), 1);
    let shown: Vec<String> = out.program.stmts.iter().map(ToSexpr::to_sexpr).collect();
    assert_eq!(shown, ["(block (print ခ))", "(print ဂ)"]);
}

#[test]
fn unclosed_block_reports_the_end_of_input()
{
    assert_eq!(codes("အောက်ပါတို့ကို လုပ်ဆောင်ပါ။ က ကို ပြသပါ။"), [Code::ExpectedToken]);
}
