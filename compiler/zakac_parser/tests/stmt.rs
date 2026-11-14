use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::Slot;
use zakac_ast::visit::Node;
use zakac_diagnostics::Code;
use zakac_rust_sugars::do_for;

use crate::common::codes;
use crate::common::parse_text;
use crate::common::stmt;
use crate::common::stmts;

#[test]
fn print_statements()
{
    assert_eq!(stmt("အသက် ကို ပြသမည်။"), "(print အသက်)");
    assert_eq!(stmt("အသက် + ၁ ကို ပြသမည်။"), "(print (+ အသက် 1))");
    assert_eq!(stmt("\"မင်္ဂလာပါ\" ဟု ပြသမည်။"), "(print \"မင်္ဂလာပါ\")");
    let logical = "(အသက် သည် ၁၈ အောက် ငယ်သည် မဟုတ်) ကို ပြသမည်။";
    assert_eq!(stmt(logical), "(print [(not (lt အသက် 18))])");
    let quoted = "(အသက် သည် ၁၈ အောက် ငယ်သည်) ဟု ပြသမည်။";
    assert_eq!(stmt(quoted), "(print [(lt အသက် 18)])");
    assert_eq!(stmt("\"x\" ကို ပြသမည်။"), "(print \"x\")");
}

#[test]
fn print_records_its_particle()
{
    let slot = |text: &str| match stmts(text).remove(0)
    {
        Stmt::Print(p) => p.slot,
        other => panic!("{other:?}"),
    };
    assert_eq!(slot("အသက် ကို ပြသမည်။"), Slot::Ko);
    assert_eq!(slot("အသက် ဟု ပြသမည်။"), Slot::Hu);
    assert_eq!(slot("\"x\" ကို ပြသမည်။"), Slot::Ko);
    assert_eq!(slot("\"x\" ဟု ပြသမည်။"), Slot::Hu);
}

#[test]
fn declarations()
{
    assert_eq!(stmt("ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။"), "(var အသက် isize)");
    assert_eq!(stmt("ကိန်းရှင် အသက် သည် ၁၈ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။"), "(var အသက် _ 18isize)");
    assert_eq!(stmt("ကိန်းသေ ပိုင် သည် ၃.၁၄ ဟူသော ဒဿမကိန်း တစ်ခု ဖြစ်ပါစေ။"), "(const ပိုင် _ 3.14)");
    let sized = "ကိန်းရှင် ရေတွက် သည် လက္ခဏာမဲ့ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(stmt(sized), "(var ရေတွက် u32)");
    let twice = "ကိန်းရှင် နှစ်ဆ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် * ၂ ဖြစ်ပါစေ။";
    assert_eq!(stmt(twice), "(var နှစ်ဆ isize (* အသက် 2))");
    let adult = "ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (အသက် သည် ၁၈ ထက် ကြီးသည်) ဖြစ်ပါစေ။";
    assert_eq!(stmt(adult), "(var ကြီးသူ bool [(gt အသက် 18)])");
    let kinded = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၅ ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(stmt(kinded), "(var က isize 5i8)");
    let clause = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ နှင့် ဂ ကို ပေါင်းသည့်ရလဒ် ဖြစ်ပါစေ။";
    assert_eq!(stmt(clause), "(var က isize (add ခ ဂ))");
}

/// The kind names the compiler gives are names, not reserved words: a kind
/// the file declares under one of them hides the compiler's, a variable can
/// take one, and one standing for a value is still reported.
#[test]
fn kind_names_are_names()
{
    let decl = "ဒဿမကိန်း ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။";
    let var = "ကိန်းရှင် က သည် ဒဿမကိန်း တစ်ခု ဖြစ်ပါစေ။";
    let list = stmts(&format!("{decl}\n{var}"));
    assert_eq!(list[1].to_sexpr(), "(var က ဒဿမကိန်း)");
    assert_eq!(stmt("ကိန်းရှင် ဒဿမကိန်း သည် ၃ ဖြစ်ပါစေ။"), "(var ဒဿမကိန်း _ 3)");
    let value = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ ကိန်းပြည့် ကို ပြသမည်။ ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(value), [Code::ExpectedOperand]);
}

/// A declaration writes its kind as `<kind> တစ်<classifier>`, or writes a value
/// alone, whose kind the checker takes.
#[test]
fn declarations_count_their_kind_or_take_it_from_the_value()
{
    let classifier = |text: &str| match &stmts(text)[0]
    {
        Stmt::Decl(d) =>
        {
            d.kind.as_ref()?.classifier.as_ref().map(|c| c.name.to_string())
        }
        other => panic!("{other:?}"),
    };
    let letter = "ကိန်းရှင် င သည် စာလုံး တစ်လုံး ဖြစ်ပါစေ။";
    assert_eq!(stmt(letter), "(var င char)");
    assert_eq!(classifier(letter).as_deref(), Some("လုံး"));
    let heap = "ကိန်းရှင် ဟ သည် စာသား စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(stmt(heap), "(var ဟ text[])");
    let named = "ကိန်းရှင် ခ သည် \"မမ\" ဖြစ်ပါစေ။";
    assert_eq!(stmt(named), "(var ခ _ \"မမ\")");
    assert_eq!(classifier(named), None);
    assert_eq!(stmt("ကိန်းရှင် ဂ သည် ခ ဖြစ်ပါစေ။"), "(var ဂ _ ခ)");
    assert_eq!(stmt("ကိန်းရှင် ဂ သည် ခ + ၁ ဖြစ်ပါစေ။"), "(var ဂ _ (+ ခ 1))");
    let person =
        "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။\n";
    let spaced = format!("{person}ကိန်းရှင် စ သည် လူ တစ်ယောက် ဖြစ်ပါစေ။");
    let joined = format!("{person}ကိန်းရှင် စ သည် လူတစ်ယောက် ဖြစ်ပါစေ။");
    assert_eq!(stmts(&spaced)[1].to_sexpr(), stmts(&joined)[1].to_sexpr());
}

/// A kind with no measure word after it is ZK0234, and the help names the word
/// the kind takes.
#[test]
fn a_kind_without_its_count_word_is_zk0234()
{
    let help = |text: &str| {
        let out = parse_text(text);
        assert_eq!(
            out.diagnostics.iter().map(|d| d.code).collect::<Vec<_>>(),
            [Code::DeclNeedsClassifier]
        );
        out.diagnostics[0].help.clone().unwrap_or_default()
    };
    assert!(help("ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။").contains("တစ်ခု"));
    assert!(help("ကိန်းရှင် က သည် စာလုံး ဖြစ်ပါစေ။").contains("တစ်လုံး"));
    let person =
        "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။\n";
    assert!(help(&format!("{person}ကိန်းရှင် က သည် လူ ဖြစ်ပါစေ။")).contains("တစ်ယောက်"));
    help("ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။");
    help("ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ ပ ဖြစ်ပါစေ။");
}

#[test]
fn declarations_with_a_value_clause()
{
    let age = "ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။";
    assert_eq!(stmt(age), "(var အသက် isize 18)");
    let spaced = "ကိန်းသေ ပိုင် သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုး သည် ၃.၁၄ ဖြစ်ပါစေ။";
    assert_eq!(stmt(spaced), "(const ပိုင် f64 3.14)");
    let clause = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ နှင့် ဂ ကို ပေါင်းသည့်ရလဒ် ဖြစ်ပါစေ။";
    assert_eq!(stmt(clause), "(var က isize (add ခ ဂ))");
    let adult = "ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီး ဖြစ်ပါစေ။";
    assert_eq!(stmt(adult), "(var ကြီးသူ bool (gt အသက် 18))");
    let bare = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ ၅ ဖြစ်ပါစေ။";
    assert_eq!(codes(bare), [Code::ExpectedToken]);
}

#[test]
fn assignments()
{
    assert_eq!(stmt("အသက် ကို ၂၄ ဟု ထားမည်။"), "(set အသက် 24)");
    assert_eq!(
        stmt("ကြီးသူ ကို (အသက် သည် ၁၈ ထက် ကြီးသည်) ဟု ထားမည်။"),
        "(set ကြီးသူ [(gt အသက် 18)])"
    );
    assert_eq!(stmt("ဝင်ခွင့်ပြုမည် ကို မမှန် ဟု ထားမည်။"), "(set ဝင်ခွင့်ပြုမည် false)");
}

#[test]
fn blocks_accept_each_closer_spelling()
{
    for closer in ["ပြီးပါပြီ", "ပြီးပါပြီး", "ပြီးဆုံးပါပြီ", "ပြီးဆုံးပါပြီး"]
    {
        let text = format!("အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nက ကို ပြသမည်။\nလုပ်ဆောင်စရာများ {closer}။");
        assert_eq!(stmt(&text), "(block (print က))", "{closer}");
    }
}

#[test]
fn conditionals()
{
    let text = "အကယ်၍ အသက် သည် ၁၈ အောက် ငယ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဝင်ခွင့်ပြုမည် ကို မမှန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ထိုသို့မဟုတ်ဘဲ အသက် သည် ၆၀ ထက် ကြီးလျှင် \"ok\" ဟု ပြသမည်။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဝင်ခွင့်ပြုမည် ကို မှန် ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(if (lt အသက် 18) (block (set ဝင်ခွင့်ပြုမည် false)) (gt အသက် 60) (block \
                (print \"ok\")) else (block (set ဝင်ခွင့်ပြုမည် true)))";
    assert_eq!(stmt(text), want);
    let then_is = "အကယ်၍ အသက် သည် ၁၈ အောက် မငယ်သည် ဖြစ်လျှင် က ကို ပြသမည်။";
    assert_eq!(stmt(then_is), "(if (ge အသက် 18) (block (print က)))");
    let not = "အကယ်၍ ဝင်ခွင့်ပြုမည် မဟုတ်လျှင် ပရိုဂရမ်ပိတ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။";
    assert_eq!(stmt(not), "(if (not ဝင်ခွင့်ပြုမည်) (block (call ပရိုဂရမ်ပိတ်)))");
}

#[test]
fn loops()
{
    let while_loop = "ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့ ကိန်း ကို ပြသမည်။";
    assert_eq!(stmt(while_loop), "(while (lt ကိန်း 5) (block (print ကိန်း)))");
    let counted = "၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။";
    assert_eq!(stmt(counted), "(count ကိန်း 1 10 count (block (print ကိန်း)))");
    let stepped = "၁၀ မှ ၀ အထိ ၂ စီ ကိန်း အနေဖြင့် လျှော့၍ ကိန်း ကို ပြသမည်။";
    assert_eq!(stmt(stepped), "(count ကိန်း 10 0 step 2 down (block (print ကိန်း)))");
    let generic = "ကိန်းရှင် ကိန်း သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပြီး ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ \
                   အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်း ကို ပြသမည်။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want =
        "(for ကိန်း isize 0 (lt ကိန်း 10) (block (print ကိန်း)) then (set ကိန်း (+ ကိန်း 1)))";
    let inferred =
        "(for ကိန်း _ 0isize (lt ကိန်း 10) (block (print ကိန်း)) then (set ကိန်း (+ ကိန်း 1)))";
    assert_eq!(stmt(generic), inferred);
    let clause = "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ \
                  အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်း ကို ပြသမည်။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(stmt(clause), want);
}

#[test]
fn break_continue_and_return()
{
    let text = "၁ မှ ၁၀၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
အကယ်၍ ကိန်း % ၂ သည် ၀ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်မည်။
အကယ်၍ ကိန်း သည် ၁၀ ထက် ကြီးလျှင် ကွင်းဆက် ကို ရပ်မည်။
ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(count ကိန်း 1 100 count (block (if (eq (% ကိန်း 2) 0) (block \
                (continue))) (if (gt ကိန်း 10) (block (break))) (return)))";
    assert_eq!(stmt(text), want);
}

#[test]
fn functions_and_calls()
{
    let text = "လုပ်ဆောင်ချက် နှစ်ဆတိုးပြီးပေါင်း သည်
က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ဂ ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း \
                လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က + ခ + ဂ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(fn နှစ်ဆတိုးပြီးပေါင်း [isize က, isize ခ, isize ဂ] isize (block (return (+ \
                (+ က ခ) ဂ))))";
    assert_eq!(stmt(text), want);
    let bare = "လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"မင်္ဂလာပါ\" ဟု ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(stmt(bare), "(fn နှုတ်ဆက် [] - (block (print \"မင်္ဂလာပါ\")))");
    let one = "လုပ်ဆောင်ချက် f သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ \
               လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(one), "(fn f [isize က] - (block))");
    let ret = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဒဿမကိန်း တစ်ခု ကို ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ \
               ပြီးပါပြီ။";
    assert_eq!(stmt(ret), "(fn f [] f64 (block))");
    let and = "လုပ်ဆောင်ချက် f သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို \
               လည်းကောင်း ဂ ဟူသော စာလုံး တစ်လုံး ကို လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ \
               မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(and), "(fn f [isize က, isize ခ, char ဂ] bool (block))");
    let two = "လုပ်ဆောင်ချက် f သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော ကိန်းပြည့် တစ်ခု ကို \
               လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(two), "(fn f [isize က, isize ခ] - (block))");
    assert_eq!(stmt("ပရိုဂရမ်ပိတ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။"), "(call ပရိုဂရမ်ပိတ်)");
    let args = "ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို အသက် နှင့် အမည် တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(stmt(args), "(call ထုတ်ပြ အသက် အမည်)");
}

#[test]
fn the_start_block_has_the_parts_of_a_function()
{
    let text = "ပရိုဂရမ်သည်
က ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း ခ ဟူသော စာသား စာရင်း တစ်ခု ကို လည်းကောင်း လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    assert_eq!(
        stmt(text),
        "(start ပရိုဂရမ် [text[] က, text[] ခ] isize (block (return 0)))"
    );
    let bare = "ပရိုဂရမ် သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    assert_eq!(stmt(bare), "(start ပရိုဂရမ် [] - (block))");
    let nested = "အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ ပရိုဂရမ် ပြီးဆုံးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(nested), [Code::NestedFunction]);
    let exported = "အများသုံး ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(exported), [Code::ExportNotOnDecl]);
    let wrong_closer = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(wrong_closer), [Code::ExpectedToken]);
    let other_spelling = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ ပရိုဂရမ် ပြီးပါပြီ။";
    assert_eq!(stmt(other_spelling), "(start ပရိုဂရမ် [] - (block))");
}

#[test]
fn statement_spans_cover_the_full_stop()
{
    let text = "က ကို ပြသမည်။ ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    let list = stmts(text);
    let out = parse_text(text);
    let end = |s: &Stmt| (s.span().hi - out.program.span.lo) as usize;
    assert_eq!(&text[..end(&list[0])], "က ကို ပြသမည်။");
    assert!(text[..end(&list[1])].ends_with("ဖြစ်ပါစေ။"));
}

#[test]
fn statement_errors()
{
    assert_eq!(codes("သို့မဟုတ်ပါက က ကို ပြသမည်။"), [Code::ElseWithoutIf]);
    assert_eq!(codes("ပြီးလျှင် က ကို ပြသမည်။"), [Code::UpdateOutsideLoop]);
    assert_eq!(codes("လုပ်ဆောင်စရာများ ပြီးပါပြီ။"), [Code::UnmatchedBlockClose]);
    let nested_if = "အကယ်၍ က လျှင် အကယ်၍ ခ လျှင် ဂ ကို ပြသမည်။";
    assert_eq!(codes(nested_if)[0], Code::NestedSingleLineIf);
    let const_loop = "ကိန်းသေ က သည် ၀ ဟူသော ကိန်းပြည့် တစ်ခု ဖြစ်ပြီး က သည် ၁ အောက် ငယ်နေသရွေ့ \
                      အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(const_loop)[0], Code::ConstLoop);
    let late = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပြီး က သည် ၁ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြီးလျှင် က ကို က + ၁ ဟု ထားမည်။
က ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(late), [Code::UpdateNotLast]);
    // A function with statements can sit in a block; an external one cannot.
    let nested_fn = "အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert!(codes(nested_fn).is_empty());
    let nested_c = "အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(nested_c), [Code::NestedFunction]);
    let no_result = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(no_result), [Code::ExpectedToken]);
    assert_eq!(codes("\"x\" ဟု ထားမည်။"), [Code::MissingOperand]);
    assert_eq!(codes("၁ မှ ၁၀ အထိ ၁ + ၁ အနေဖြင့် ရေတွက်၍ က ကို ပြသမည်။"), [Code::ExpectedToken]);
}

#[test]
fn each_loops()
{
    let cells = "l ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသမည်။";
    assert_eq!(stmt(cells), "(each_cell x l (block (print x)))");
    let chars = "t ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသမည်။";
    assert_eq!(stmt(chars), "(each_char c t (block (print c)))");
}

#[test]
fn counted_variable_joined_to_its_particle_gets_a_hint()
{
    let out = parse_text("၁ မှ ၁၀ အထိ ကိန်းအနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။");
    assert_eq!(out.diagnostics.len(), 1);
    let help = out.diagnostics[0].help.clone().unwrap_or_default();
    assert!(help.contains("ကိန်း အနေဖြင့်"), "{help}");
}

#[test]
fn errors_recover_at_the_next_statement()
{
    let text = "က ကို ကို ပြသမည်။
ခ ကို ပြသမည်။
အကယ်၍ + လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဂ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
သို့မဟုတ်ပါက ဃ ကို ပြသမည်။
င ကို ပြသမည်။";
    let out = parse_text(text);
    let found: Vec<Code> = out.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, [Code::DoubleParticle, Code::ExpectedOperand]);
    let shown: Vec<String> = out.program.stmts.iter().map(ToSexpr::to_sexpr).collect();
    assert_eq!(shown, ["(print ခ)", "(print င)"]);
}

#[test]
fn block_errors_do_not_leak()
{
    let text = "အောက်ပါတို့ကို လုပ်ဆောင်မည်။
က ကို ကို ပြသမည်။
ခ ကို ပြသမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
ဂ ကို ပြသမည်။";
    let out = parse_text(text);
    assert_eq!(out.diagnostics.len(), 1);
    let shown: Vec<String> = out.program.stmts.iter().map(ToSexpr::to_sexpr).collect();
    assert_eq!(shown, ["(block (print ခ))", "(print ဂ)"]);
}

#[test]
fn unclosed_block_reports_the_end_of_input()
{
    assert_eq!(codes("အောက်ပါတို့ကို လုပ်ဆောင်မည်။ က ကို ပြသမည်။"), [Code::ExpectedToken]);
}

#[test]
fn pointer_declarations_and_places()
{
    assert_eq!(stmt("ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var p isize*)");
    assert_eq!(stmt("ကိန်းရှင် pp သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var pp isize**)");
    assert_eq!(stmt("ကိန်းရှင် b သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var b i8*)");
    assert_eq!(stmt("ကိန်းရှင် q သည် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var q ptr)");
    assert_eq!(stmt("ကိန်းရှင် qq သည် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var qq ptr*)");
    assert_eq!(
        stmt("ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။"),
        "(var p isize* (addr x))"
    );
    assert_eq!(
        stmt("ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။"),
        "(var p isize* (addr x))"
    );
    assert_eq!(stmt("p ၏ တန်ဖိုး ကို ၅ ဟု ထားမည်။"), "(set (deref p) 5)");
    assert_eq!(stmt("*p ကို *p + ၁ ဟု ထားမည်။"), "(set (deref p) (+ (deref p) 1))");
    assert_eq!(stmt("၅ ကို ၆ ဟု ထားမည်။"), "(set 5 6)"); // the checker rejects it, not the parser
}

#[test]
fn pointer_kinds_keep_the_older_spelling()
{
    assert_eq!(stmt("ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var p isize*)");
    assert_eq!(
        stmt("ကိန်းရှင် pp သည် ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"),
        "(var pp isize**)"
    );
    assert_eq!(
        stmt("ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။"),
        "(var p isize* (addr x))"
    );
}

/// The arguments-first call and the short call build the tree that the
/// other call forms build.
#[test]
fn arguments_first_calls_match_the_other_forms()
{
    let pairs = [
        (
            "က နှင့် ခ တို့ကို သုံး၍ ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။",
            "ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်မည်။",
        ),
        ("က နှင့် ခ တို့ကိုသုံး၍ ထုတ်ပြ မည်။", "ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်မည်။"),
        ("က ကို သုံး၍ ထုတ်ပြ မည်။", "ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်မည်။"),
        ("ထုတ်ပြ မည်။", "ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။"),
        ("ထုတ်ပြ မည့် လုပ်ဆောင်ချက်ကိုလုပ်ဆောင်မည်။", "ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။"),
        (
            "က ကို သုံး၍ ထုတ်ပြ မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။",
            "ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို က ဖြင့် လုပ်ဆောင်မည်၊ မအောင်မြင်ပါက အမှားအယွင်းကို ပြန်ပေးမည်။",
        ),
        ("က ကို သုံး၍ ထုတ်ပြ သည့် ရလဒ် ကို ပြသမည်။", "ထုတ်ပြ(က) ကို ပြသမည်။"),
        ("က ကို သုံး၍ ထုတ်ပြ သည့်ရလဒ်ကောင်း ကို ပြသမည်။", "ထုတ်ပြ(က)? ကို ပြသမည်။"),
        ("က ကို သုံး၍ ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။", "ထုတ်ပြ(က) ကို ပြသမည်။"),
        ("ထုတ်ပြ သည့် ရလဒ် ကို ပြသမည်။", "ထုတ်ပြ() ကို ပြသမည်။"),
        ("ထုတ်ပြ ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ်ကောင်း ကို ပြသမည်။", "ထုတ်ပြ()? ကို ပြသမည်။"),
        ("က နှင့် ခ တို့ကို သုံး၍ တိုး မည်။", "တိုး ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်မည်။"),
        (
            "ခုနစ် သည့် ရလဒ် နှင့် က + ၁ တို့ကို သုံး၍ ပေါင်း မည်။",
            "ပေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ခုနစ်() နှင့် က + ၁ တို့ဖြင့် လုပ်ဆောင်မည်။",
        ),
        ("ကိန်းရှင် ခ သည် က ကို သုံး၍ စစ် သည့် ရလဒ်ကောင်း ဖြစ်ပါစေ။", "ကိန်းရှင် ခ သည် စစ်(က)? ဖြစ်ပါစေ။"),
    ];
    do_for!((new, old) in pairs, assert_eq!(stmt(new), stmt(old), "{new}"));
}

/// A statement end of a call stands where a value is needed alone, and the
/// long form ends in `လုပ်ဆောင်မည်`, not `လုပ်ဆောင်မည်`.
#[test]
fn call_ends_follow_the_place_of_the_call()
{
    let wrong = ["ကိန်းရှင် x သည် က ကို သုံး၍ ပြ မည် ဖြစ်ပါစေ။", "က ကို သုံး၍ ပြ။"];
    do_for!(text in wrong, assert_eq!(codes(text), [Code::ExpectedToken], "{text}"));
}

/// A parameter the function can change takes `ဒေတာ ပြောင်းလဲရန်` in its item, and
/// the call writes the argument with no mark.
#[test]
fn edit_parameters_parse_and_calls_bear_no_mark()
{
    let decl = stmt(
        "လုပ်ဆောင်ချက် တိုး သည်
န ဟူသော ကိန်းပြည့် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    );
    assert!(decl.contains("edit isize န"), "{decl}");
    let call = stmt("တိုး ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်မည်။");
    assert_eq!(call, "(call တိုး က ခ)");
}

/// The markers of an item come in order: owned, changed, then the preposition.
#[test]
fn item_markers_parse_in_order()
{
    let decl = stmt(
        "လုပ်ဆောင်ချက် ယူ သည်
ပ ဟူသော ကိန်းပြည့် လိပ်စာ တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် အပိုင်အနေဖြင့် ဝိဘတ် \"ကို\" ကို သုံး၍ လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    );
    assert!(decl.contains("own edit "), "{decl}");
    assert!(decl.contains(" ပ(ကို)"), "{decl}");
}

/// A preposition string holds words a preposition can hold.
#[test]
fn a_preposition_holds_words_a_preposition_can_hold()
{
    let header = |preposition: &str| {
        format!(
            "လုပ်ဆောင်ချက် ရေး သည် စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"{preposition}\" ကို သုံး၍ လက်ခံပြီး \
             အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။"
        )
    };
    // a preposition of several words, and a glued preposition that lexes as
    // several
    assert_eq!(codes(&header("က ခ")), []);
    assert_eq!(codes(&header("ရှိ ဖိုင်ကို")), []);
    assert_eq!(codes(&header("၏")), []);
    // a call end, a closer, a sign, and `၏` after the first word
    assert_eq!(codes(&header("ခ မည်")), [Code::BadPreposition]);
    assert_eq!(codes(&header("သည်")), [Code::BadPreposition]);
    assert_eq!(codes(&header("+")), [Code::BadPreposition]);
    assert_eq!(codes(&header("က ၏")), [Code::BadPreposition]);
    // `ကို သုံး၍` cannot start the first preposition: it starts the arguments-first
    // call
    assert_eq!(codes(&header("ကိုသုံး၍")), [Code::BadPreposition]);
}

/// A preposition that starts with `၏` goes on the last parameter alone.
#[test]
fn an_ei_preposition_goes_on_the_last_parameter()
{
    let text = "လုပ်ဆောင်ချက် ရေး သည် က ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"၏\" ကို သုံး၍ လည်းကောင်း ခ ဟူသော \
                စာသား တစ်ခု ကို ဝိဘတ် \"သို့\" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(text), [Code::PrepositionEiNotLast]);
}

/// A list with no count is plain; a written count makes a counted list.
#[test]
fn list_forms_parse()
{
    assert_eq!(stmt("ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။"), "(var l isize[])");
    assert_eq!(
        stmt("ကိန်းရှင် l သည် အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်ပါစေ။"),
        "(var l isize[3])"
    );
}

#[test]
fn add_and_remove_parse()
{
    assert_eq!(stmt("l ထဲသို့ ၃ ကို ထည့်မည်။"), "(add l 3)");
    assert_eq!(stmt("s ၏ l ထဲသို့ x + ၁ ကို ထည့်မည်။"), "(add (field s l) (+ x 1))");
    assert_eq!(stmt("l ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။"), "(remove l)");
    assert_eq!(stmt("l ၏ အကွက် ၀ ၏ နောက်ဆုံးအကွက်ကို ဖျက်မည်။"), "(remove (cell l 0))");
    assert_eq!(stmt("t ၏ နောက်ဆုံးစာလုံးကို ဖျက်မည်။"), "(remove_char t)");
}

#[test]
fn adds_in_front_of_a_cell_or_a_character_parse()
{
    let cell = stmt("l ၏ အကွက် ၁ ၏ အရှေ့တွင် ၉ ကို ထည့်မည်။");
    assert!(cell.starts_with("(insert (cell l 1") && cell.ends_with(" 9)"), "{cell}");
    let joined = stmt("l ၏ အကွက် ၁ ၏အရှေ့တွင် ၉ ကို ထည့်မည်။");
    assert_eq!(joined, cell);
    let nth = stmt("l ၏ ၂ ခုမြောက်အကွက် ၏ အရှေ့တွင် ၉ ကို ထည့်မည်။");
    assert!(nth.starts_with("(insert "), "{nth}");
    let text = stmt("t ၏ စာလုံး ၀ ၏ အရှေ့တွင် \"x\" ကို ထည့်မည်။");
    assert!(text.starts_with("(insert ") && text.contains("t"), "{text}");
}

#[test]
fn replace_of_a_range_parses()
{
    let cells = stmt("t ၏ အကွက် ၀ မှ ၄ ထိကို \"bye\" ဖြင့် အစားထိုးမည်။");
    assert_eq!(cells, "(replace t 0 4 \"bye\")");
    let chars = stmt("t ၏ ၁ လုံးမြောက်စာလုံးမှ i + ၁ လုံးမြောက်စာလုံးထိကို c ဖြင့် အစားထိုးမည်။");
    assert_eq!(chars, "(replace_chars t 1 (+ i 1) c)");
    let spaced = stmt("t ၏ ၁ ခုမြောက်စာလုံး မှ ၂ ခုမြောက်စာလုံး ထိ ကို c ဖြင့် အစားထိုးမည်။");
    assert_eq!(spaced, "(replace_chars t 1 2 c)");
    let field = stmt("s ၏ t ၏ အကွက် ၀ မှ ၁ ထိကို u ၏ အကွက် ၀ ဖြင့် အစားထိုးမည်။");
    assert!(field.starts_with("(replace (field s t) 0 1 (cell u 0"), "{field}");
}

#[test]
fn a_range_stands_in_a_replace_statement_alone()
{
    let printed = codes("t ၏ အကွက် ၀ မှ ၄ ထိကို ပြသမည်။");
    assert_eq!(printed, [Code::SlotMismatch]);
    let missing = codes("t ၏ အကွက် ၀ မှ ၄ ထိကို \"x\" ကို အစားထိုးမည်။");
    assert_eq!(missing, [Code::SlotMismatch]);
}

#[test]
fn a_counted_loop_from_a_cell_still_parses()
{
    let looped = stmt("l ၏ အကွက် ၀ မှ ၅ အထိ i အနေဖြင့် ရေတွက်၍ i ကို ပြသမည်။");
    assert!(looped.starts_with("(count i (cell l 0) 5 count"), "{looped}");
}

/// A parameter carries its preposition in parentheses after its name; with
/// prepositions the parameters end in `လက်ခံပြီး` straight after the last.
#[test]
fn parameters_have_prepositions()
{
    let text = "လုပ်ဆောင်ချက် ရေး သည် စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း နေရာ \
                ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"သို့\" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ \
                လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(text), "(fn ရေး [text စာ(ကို), text နေရာ(သို့)] - (block))");
    let mixed = "လုပ်ဆောင်ချက် ရေး သည် စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း နေရာ \
                 ဟူသော စာသား တစ်ခု ကို လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ \
                 ပြီးပါပြီ။";
    assert_eq!(codes(mixed), [Code::MixedPrepositions]);
}

/// The function `ရေး`, whose two parameters have prepositions, before `rest`.
fn with_write(rest: &str) -> String
{
    format!(
        "လုပ်ဆောင်ချက် ရေး သည် စာ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း နေရာ ဟူသော \
         စာသား တစ်ခု ကို ဝိဘတ် \"သို့\" ကို သုံး၍ လည်းကောင်း လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ \
         ပြီးပါပြီ။\n{rest}"
    )
}

/// The second statement of a program, as a tree, and its call prepositions.
fn second(text: &str) -> (String, Vec<String>)
{
    let list = stmts(text);
    let prepositions =
        |m: &[zakac_ast::expr::Ident]| m.iter().map(|i| i.name.to_string()).collect();
    let found = match &list[1]
    {
        Stmt::Call(n) => prepositions(&n.prepositions),
        Stmt::Decl(d) => match d.init.as_ref()
        {
            Some(zakac_ast::expr::Expr::Call(n)) => prepositions(&n.prepositions),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    (list[1].to_sexpr(), found)
}

/// A preposition call gives each argument with its preposition and builds the
/// call the other forms build.
#[test]
fn preposition_calls_match_the_other_forms()
{
    let want = |old: &str| second(&with_write(old)).0;
    let prepositions = vec!["ကို".to_string(), "သို့".to_string()];
    let short = second(&with_write("က ကို ခ သို့ ရေး မည်။"));
    assert_eq!(
        short,
        (want("ရေး ဟူသော လုပ်ဆောင်ချက်ကို က နှင့် ခ တို့ဖြင့် လုပ်ဆောင်မည်။"), prepositions.clone())
    );
    let long = second(&with_write("က ကို ခ သို့ ရေး ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်မည်။"));
    assert_eq!(long.1, prepositions);
    let string = second(&with_write("\"a\" ကို \"b\" သို့ ရေး မည်။"));
    assert_eq!(string.1, prepositions);
    let value = second(&with_write("ကိန်းရှင် ဂ သည် က ကို ခ သို့ ရေး သည့် ရလဒ် ဖြစ်ပါစေ။"));
    assert_eq!(value.1, prepositions);
}

/// A name that declares no prepositions ends no preposition call, and a call
/// whose prepositions do not fit its function reads as the sentence it was
/// before.
#[test]
fn other_sentences_keep_their_meaning()
{
    let set = "ကိန်းရှင် က သည် ၀ ဖြစ်ပါစေ။\nက ကို ခုနစ် သည့် ရလဒ် ဟု ထားမည်။";
    assert!(stmts(set)[1].to_sexpr().starts_with("(set က"), "{set}");
    let read = "လုပ်ဆောင်ချက် ဖတ် သည် လမ်း ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \"၌ရှိသော\" ကို သုံး၍ လက်ခံပြီး \
                အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။\nက ကို ခ \
                ၌ရှိသော ဖတ် သည့် ရလဒ် ဟု ထားမည်။";
    assert!(stmts(read)[1].to_sexpr().starts_with("(set က"), "{read}");
    let import = format!("\"lib.zaka\" မှ ရေး ကို ထည့်သွင်းမည်။\n{}", with_write(""));
    assert!(matches!(stmts(&import)[0], Stmt::Import(_)));
}

/// A function kind with two text parameters and no prepositions.
const PLAIN_KIND: &str =
    "စာသား တစ်ခု ကို လည်းကောင်း စာသား တစ်ခု ကို လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား";

/// The function kind of `ရေး`, with its prepositions.
const WRITE_KIND: &str = "(စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လည်းကောင်း စာသား တစ်ခု ကို ဝိဘတ် \"သို့\" \
                          ကို သုံး၍ လည်းကောင်း လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား)";

/// A declaration that writes a kind gives a variable the prepositions of that
/// kind, whatever its starting value; with no kind written, the starting
/// function gives them.
#[test]
fn a_written_kind_gives_the_prepositions_of_a_variable()
{
    let call = "\"က\" ကို \"ခ\" သို့ ဖ မည်။";
    let plain = |word: &str| {
        with_write(&format!(
            "{word} ဖ သည် {PLAIN_KIND} တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ရေး ဟူသော လုပ်ဆောင်ချက် \
             ဖြစ်ပါစေ။\n{call}"
        ))
    };
    assert_eq!(codes(&plain("ကိန်းရှင်")), [Code::UnclosedClause]);
    assert_eq!(codes(&plain("ကိန်းသေ")), [Code::UnclosedClause]);
    let bare = with_write(&format!("ကိန်းရှင် ဖ သည် ရေး ဟူသော လုပ်ဆောင်ချက် ဖြစ်ပါစေ။\n{call}"));
    assert!(matches!(stmts(&bare)[2], Stmt::Call(_)), "{bare}");
    let blank = with_write(&format!(
        "ကိန်းရှင် ဖ သည် {WRITE_KIND} တစ်ခု ဖြစ်ပါစေ။\nဖ ကို ရေး ဟူသော လုပ်ဆောင်ချက် ဟု ထားမည်။\n{call}"
    ));
    assert!(matches!(stmts(&blank)[3], Stmt::Call(_)), "{blank}");
}

/// The header of an external function, in each shape: no parameters and no
/// result, parameters with or without a result, or a result alone; with or
/// without `တစ်ခု`, glued or apart.
#[test]
fn external_functions_have_a_header_alone()
{
    let bare = "လုပ်ဆောင်ချက် က သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်တစ်ခုဖြစ်သည်။";
    assert_eq!(stmt(bare), "(external က [] -)");
    let apart = "လုပ်ဆောင်ချက် က သည် ပြင်ပ မှ လာသော လုပ်ဆောင်ချက် တစ်ခု ဖြစ်သည်။";
    assert_eq!(stmt(apart), "(external က [] -)");
    let takes = "လုပ်ဆောင်ချက် ရေး သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စ ဟူသော စာသား တစ်ခု ကို ဝိဘတ် \
                 \"ကို\" ကို သုံး၍ လက်ခံမည်။";
    assert_eq!(stmt(takes), "(external ရေး [text စ(ကို)] -)");
    let gives = "လုပ်ဆောင်ချက် ဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ စာသား တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(stmt(gives), "(external ဖတ် [] text)");
    let both = "လုပ်ဆောင်ချက် ဖတ် သည် ပြင်ပမှလာသော လုပ်ဆောင်ချက်တစ်ခု ဖြစ်၍ လမ်း ဟူသော စာသား တစ်ခု ကို \
                လက်ခံပြီး စာသား တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(stmt(both), "(external ဖတ် [text လမ်း] text)");
}

#[test]
fn c_functions_name_their_lib()
{
    let abs = "လုပ်ဆောင်ချက် abs သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ n ဟူသော i32 တစ်ခု ကို လက်ခံပြီး \
               i32 တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(stmt(abs), "(external \"libc\" abs [i32 n] i32)");
    let apart = "လုပ်ဆောင်ချက် rand သည် \"libc\" မှ လာသော လုပ်ဆောင်ချက်ဖြစ်၍ i32 တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(stmt(apart), "(external \"libc\" rand [] i32)");
    let printf = "လုပ်ဆောင်ချက် printf သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ f ဟူသော စာသား တစ်ခု \
                  ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး i32 တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(stmt(printf), "(external \"libc\" printf [text f, ...] i32)");
    let listed = "လုပ်ဆောင်ချက် printf သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ f ဟူသော စာသား တစ်ခု \
                  ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး i32 တစ်ခု ကို ပြန်ပေးမည်။";
    assert_eq!(stmt(listed), "(external \"libc\" printf [text f, ...] i32)");
    let spaced = listed.replace("များနှင့်", "များ နှင့်");
    assert_eq!(stmt(&spaced), "(external \"libc\" printf [text f, ...] i32)");
}

#[test]
fn a_c_function_can_give_back_its_text_as_the_program_s_own()
{
    let owned = "လုပ်ဆောင်ချက် strdup သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍ s ဟူသော စာသား တစ်ခု ကို \
                 လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို အပိုင်အနေဖြင့် ပြန်ပေးမည်။";
    let list = stmts(owned);
    let zakac_ast::stmt::Stmt::Fn(f) = &list[0]
    else
    {
        panic!("a function");
    };
    assert!(f.owned_ret);
    let kept = owned.replace("အပိုင်အနေဖြင့် ", "");
    let zakac_ast::stmt::Stmt::Fn(f) = &stmts(&kept)[0]
    else
    {
        panic!("a function");
    };
    assert!(!f.owned_ret);
}

/// A short parameter list whose one parameter is a function kind with
/// prepositions: the `လည်းကောင်း` inside the parentheses belongs to the function
/// kind, not to the parameter list.
#[test]
fn a_function_kind_with_prepositions_stands_in_a_short_list()
{
    let text = "လုပ်ဆောင်ချက် ခေါ် သည် ဖ ဟူသော (စာသား တစ်ခု ကို ဝိဘတ် \"ကို\" ကို သုံး၍ လက်ခံသော လုပ်ဆောင်ချက် \
                အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(text), []);
}

/// `<value> ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။` gives code outside the program leave
/// to call a function value; `ပြန်လည်ရုပ်သိမ်းမည်` takes it back. `C မှ` is no
/// such statement.
#[test]
fn a_leave_for_outside_code_names_its_value()
{
    let grant = stmt("ဖ ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြုမည်။");
    assert_eq!(grant, "(c_leave ဖ)");
    let fused = stmt("ဖ ကို ပြင်ပမှ ခေါ်ဆိုခွင့်ပြုမည်။");
    assert_eq!(fused, grant);
    let revoke = stmt("ထည့် ဟူသော လုပ်ဆောင်ချက်ကို ပြင်ပမှ ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်။");
    assert_eq!(revoke, "(c_revoke (fn_addr ထည\u{103a}\u{1037}))");
    assert!(!codes("ဖ ကို C မှ ခေါ်ဆိုခွင့် ပြုမည်။").is_empty());
}

/// A kind from outside the program, with no fields and with C fields.
#[test]
fn a_kind_from_outside_parses()
{
    let bare = stmt("စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်သည်။");
    assert_eq!(bare, "(external_struct စက\u{103a} [])");
    let fields =
        stmt("စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် n ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။");
    assert_eq!(fields, "(external_struct စက\u{103a} [i32 n])");
    // `အများသုံး` stands right before the noun, after the external mark.
    let shown = "စက် ဟူသော ပြင်ပမှ လာသော အများသုံး ဒေတာအစုအဝေး ဖြစ်သည်။";
    assert_eq!(codes(shown), []);
    assert!(stmt(shown).contains("export"), "{}", stmt(shown));
    assert!(!codes("စက် ဟူသော အများသုံး ပြင်ပမှ လာသော ဒေတာအစုအဝေး ဖြစ်သည်။").is_empty());
}

/// A function kind takes `ဒေတာ ပြောင်းလဲရန်` alone; `ဒေတာ ထည့်ပေးရန်` on one of
/// its parameters is reported at the mark.
#[test]
fn a_filled_parameter_in_a_function_kind_is_zk0243()
{
    let text = "လုပ်ဆောင်ချက် သုံး သည်
ဖ ဟူသော (ကိန်းပြည့် တစ်ခု ကို ဒေတာ ထည့်ပေးရန် လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(text), [Code::FillInFnKind]);
}

/// One field or one member closes with `ပါဝင်သည်`; `တို့ပါဝင်သည်` is for several.
#[test]
fn one_member_closes_without_toh()
{
    let one = "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။";
    assert_eq!(codes(one), []);
    let named =
        "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။";
    assert_eq!(codes(named), []);
    let colour = "အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် အနီ ပါဝင်သည်။";
    assert_eq!(codes(colour), []);
    let several =
        "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် အရပ် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။";
    assert_eq!(codes(several), []);
    let toh_one = [
        "လူ ဟူသော ဒေတာအစုအဝေးတွင် အသက် ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။",
        "အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် အနီ တို့ပါဝင်သည်။",
        "ဖိုင် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် fd ဟူသော ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။",
    ];
    do_for!(text in toh_one, assert_eq!(codes(text), [Code::ExpectedToken], "{text}"));
    let bare_several = "အရောင် ဟူသော ဒေတာအမျိုးအစားကွဲတွင် အနီ နှင့် အစိမ်း ပါဝင်သည်။";
    assert_eq!(codes(bare_several), [Code::ExpectedToken]);
}
