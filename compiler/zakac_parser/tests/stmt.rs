mod common;

use common::codes;
use common::parse_text;
use common::stmt;
use common::stmts;
use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Stmt;
use zakac_ast::verb::Slot;
use zakac_ast::visit::Node;
use zakac_diagnostics::Code;

#[test]
fn print_statements()
{
    assert_eq!(stmt("အသက် ကို ပြသပါ။"), "(print အသက်)");
    assert_eq!(stmt("အသက် + ၁ ကို ပြသပါ။"), "(print (+ အသက် 1))");
    assert_eq!(stmt("\"မင်္ဂလာပါ\" ဟု ပြသပါ။"), "(print \"မင်္ဂလာပါ\")");
    let logical = "(အသက် သည် ၁၈ အောက် ငယ်သည် မဟုတ်) ကို ပြသပါ။";
    assert_eq!(stmt(logical), "(print [(not (lt အသက် 18))])");
    let quoted = "(အသက် သည် ၁၈ အောက် ငယ်သည်) ဟု ပြသပါ။";
    assert_eq!(stmt(quoted), "(print [(lt အသက် 18)])");
    assert_eq!(stmt("\"x\" ကို ပြသပါ။"), "(print \"x\")");
}

#[test]
fn print_records_its_particle()
{
    let slot = |text: &str| match stmts(text).remove(0)
    {
        Stmt::Print(p) => p.slot,
        other => panic!("{other:?}"),
    };
    assert_eq!(slot("အသက် ကို ပြသပါ။"), Slot::Ko);
    assert_eq!(slot("အသက် ဟု ပြသပါ။"), Slot::Hu);
    assert_eq!(slot("\"x\" ကို ပြသပါ။"), Slot::Ko);
    assert_eq!(slot("\"x\" ဟု ပြသပါ။"), Slot::Hu);
}

#[test]
fn declarations()
{
    assert_eq!(stmt("ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။"), "(var အသက် isize)");
    assert_eq!(stmt("ကိန်းရှင် အသက် သည် ကိန်းပြည့် ၁၈ ဖြစ်ပါစေ။"), "(var အသက် _ 18isize)");
    assert_eq!(stmt("ကိန်းသေ ပိုင် သည် ဒဿမကိန်း ၃.၁၄ ဖြစ်ပါစေ။"), "(const ပိုင် _ 3.14)");
    let sized = "ကိန်းရှင် ရေတွက် သည် လက္ခဏာမဲ့ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(stmt(sized), "(var ရေတွက် u32)");
    let twice = "ကိန်းရှင် နှစ်ဆ သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် * ၂ ဖြစ်ပါစေ။";
    assert_eq!(stmt(twice), "(var နှစ်ဆ isize (* အသက် 2))");
    let adult = "ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (အသက် သည် ၁၈ ထက် ကြီးသည်) ဖြစ်ပါစေ။";
    assert_eq!(stmt(adult), "(var ကြီးသူ bool [(gt အသက် 18)])");
    let kinded = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၈ ဘစ် ကိန်းပြည့် ၅ ဖြစ်ပါစေ။";
    assert_eq!(stmt(kinded), "(var က isize 5i8)");
    let clause = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ နှင့် ဂ ၏ ပေါင်းလဒ် ဖြစ်ပါစေ။";
    assert_eq!(stmt(clause), "(var က isize (add ခ ဂ))");
}

/// A declaration writes its kind as `<kind> တစ်<classifier>`, or writes a value
/// alone, whose kind the checker takes.
#[test]
fn declarations_count_their_kind_or_take_it_from_the_value()
{
    let classifier = |text: &str| match &stmts(text)[0]
    {
        Stmt::Decl(d) => d.classifier.as_ref().map(|c| c.name.to_string()),
        other => panic!("{other:?}"),
    };
    let letter = "ကိန်းရှင် င သည် စာလုံး တစ်လုံး ဖြစ်ပါစေ။";
    assert_eq!(stmt(letter), "(var င char)");
    assert_eq!(classifier(letter).as_deref(), Some("လုံး"));
    let fixed = "ကိန်းရှင် ဖ သည် စာသားသေ တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(stmt(fixed), "(var ဖ fixed_text)");
    let heap = "ကိန်းရှင် ဟ သည် စာသားရှင် စာရင်း တစ်ခု ဖြစ်ပါစေ။";
    assert_eq!(stmt(heap), "(var ဟ []heap_text)");
    let named = "ကိန်းရှင် ခ သည် \"မမ\" ဖြစ်ပါစေ။";
    assert_eq!(stmt(named), "(var ခ _ \"မမ\")");
    assert_eq!(classifier(named), None);
    assert_eq!(stmt("ကိန်းရှင် ဂ သည် ခ ဖြစ်ပါစေ။"), "(var ဂ _ ခ)");
    assert_eq!(stmt("ကိန်းရှင် ဂ သည် ခ + ၁ ဖြစ်ပါစေ။"), "(var ဂ _ (+ ခ 1))");
    let person = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။\n";
    let spaced = format!("{person}ကိန်းရှင် စ သည် လူ တစ်ယောက် ဖြစ်ပါစေ။");
    let joined = format!("{person}ကိန်းရှင် စ သည် လူတစ်ယောက် ဖြစ်ပါစေ။");
    assert_eq!(stmts(&spaced)[1].to_sexpr(), stmts(&joined)[1].to_sexpr());
}

/// A kind with no count word after it is the form of old, and the help names
/// the count word the kind takes.
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
    let person = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။\n";
    assert!(help(&format!("{person}ကိန်းရှင် က သည် လူ ဖြစ်ပါစေ။")).contains("တစ်ယောက်"));
    help("ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၅ ဖြစ်ပါစေ။");
    help("ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ ပ ဖြစ်ပါစေ။");
    help("ကိန်းရှင် က သည် ငှားထားသော ကိန်းပြည့် ၅ ဖြစ်ပါစေ။");
}

#[test]
fn declarations_with_a_value_clause()
{
    let age = "ကိန်းရှင် အသက် သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၁၈ ဖြစ်ပါစေ။";
    assert_eq!(stmt(age), "(var အသက် isize 18)");
    let spaced = "ကိန်းသေ ပိုင် သည် ဒဿမကိန်း တစ်ခု ဖြစ်၍ တန်ဖိုး သည် ၃.၁၄ ဖြစ်ပါစေ။";
    assert_eq!(stmt(spaced), "(const ပိုင် f64 3.14)");
    let clause = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ခ နှင့် ဂ ၏ ပေါင်းလဒ် ဖြစ်ပါစေ။";
    assert_eq!(stmt(clause), "(var က isize (add ခ ဂ))");
    let adult = "ကိန်းရှင် ကြီးသူ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် အသက် သည် ၁၈ ထက် ကြီး ဖြစ်ပါစေ။";
    assert_eq!(stmt(adult), "(var ကြီးသူ bool (gt အသက် 18))");
    let bare = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ ၅ ဖြစ်ပါစေ။";
    assert_eq!(codes(bare), [Code::ExpectedToken]);
}

#[test]
fn assignments()
{
    assert_eq!(stmt("အသက် ကို ၂၄ ဟု ထားပါ။"), "(set အသက် 24)");
    assert_eq!(
        stmt("ကြီးသူ ကို (အသက် သည် ၁၈ ထက် ကြီးသည်) ဟု ထားပါ။"),
        "(set ကြီးသူ [(gt အသက် 18)])"
    );
    assert_eq!(stmt("ဝင်ခွင့်ပြုမည် ကို မမှန် ဟု ထားပါ။"), "(set ဝင်ခွင့်ပြုမည် false)");
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
ဝင်ခွင့်ပြုမည် ကို မမှန် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
ထိုသို့မဟုတ်ဘဲ အသက် သည် ၆၀ ထက် ကြီးလျှင် \"ok\" ဟု ပြသပါ။
သို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ဝင်ခွင့်ပြုမည် ကို မှန် ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(if (lt အသက် 18) (block (set ဝင်ခွင့်ပြုမည် false)) (gt အသက် 60) (block \
                (print \"ok\")) else (block (set ဝင်ခွင့်ပြုမည် true)))";
    assert_eq!(stmt(text), want);
    let then_is = "အကယ်၍ အသက် သည် ၁၈ အောက် မငယ်သည် ဖြစ်လျှင် က ကို ပြသပါ။";
    assert_eq!(stmt(then_is), "(if (ge အသက် 18) (block (print က)))");
    let not = "အကယ်၍ ဝင်ခွင့်ပြုမည် မဟုတ်လျှင် လုပ်ဆောင်ချက် ပရိုဂရမ်ပိတ် ကို လုပ်ဆောင်ပါ။";
    assert_eq!(stmt(not), "(if (not ဝင်ခွင့်ပြုမည်) (block (call ပရိုဂရမ်ပိတ်)))");
}

#[test]
fn loops()
{
    let while_loop = "ကိန်း သည် ၅ အောက် ငယ်နေသရွေ့ ကိန်း ကို ပြသပါ။";
    assert_eq!(stmt(while_loop), "(while (lt ကိန်း 5) (block (print ကိန်း)))");
    let counted = "၁ မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။";
    assert_eq!(stmt(counted), "(count ကိန်း 1 10 count (block (print ကိန်း)))");
    let stepped = "၁၀ မှ ၀ အထိ ၂ စီ ကိန်း အနေဖြင့် လျှော့၍ ကိန်း ကို ပြသပါ။";
    assert_eq!(stmt(stepped), "(count ကိန်း 10 0 step 2 down (block (print ကိန်း)))");
    let generic = "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို \
                   လုပ်ဆောင်ပါ။
ကိန်း ကို ပြသပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want =
        "(for ကိန်း isize 0 (lt ကိန်း 10) (block (print ကိန်း)) then (set ကိန်း (+ ကိန်း 1)))";
    let inferred =
        "(for ကိန်း _ 0isize (lt ကိန်း 10) (block (print ကိန်း)) then (set ကိန်း (+ ကိန်း 1)))";
    assert_eq!(stmt(generic), inferred);
    let clause = "ကိန်းရှင် ကိန်း သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ ကိန်း သည် ၁၀ အောက် ငယ်နေသရွေ့ \
                  အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
ကိန်း ကို ပြသပါ။
ပြီးလျှင် ကိန်း ကို ကိန်း + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(stmt(clause), want);
}

#[test]
fn break_continue_and_return()
{
    let text = "၁ မှ ၁၀၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
အကယ်၍ ကိန်း % ၂ သည် ၀ နှင့် ညီလျှင် နောက်တစ်ကြိမ်ဆီသို့ ကျော်ပါ။
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
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ၊ ကိန်းပြည့် ဂ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က + ခ + ဂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    let want = "(fn နှစ်ဆတိုးပြီးပေါင်း [isize က, isize ခ, isize ဂ] isize (block (return (+ \
                (+ က ခ) ဂ))))";
    assert_eq!(stmt(text), want);
    let bare = "လုပ်ဆောင်ချက် နှုတ်ဆက် သည်
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"မင်္ဂလာပါ\" ဟု ပြသပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။";
    assert_eq!(stmt(bare), "(fn နှုတ်ဆက် [] - (block (print \"မင်္ဂလာပါ\")))");
    let one =
        "လုပ်ဆောင်ချက် f သည် ကိန်းပြည့် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(one), "(fn f [isize က] - (block))");
    let ret = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဒဿမကိန်း ရလဒ် ကို ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ \
               ပြီးပါပြီ။";
    assert_eq!(stmt(ret), "(fn f [] f64 (block))");
    let and = "လုပ်ဆောင်ချက် f သည် ကိန်းပြည့် က၊ ကိန်းပြည့် ခ နှင့် စာလုံး ဂ တို့ကို လက်ခံပြီး အောက်ပါတို့ကို \
               လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(and), "(fn f [isize က, isize ခ, char ဂ] bool (block))");
    let two = "လုပ်ဆောင်ချက် f သည် ကိန်းပြည့် က နှင့် ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။ \
               လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(two), "(fn f [isize က, isize ခ] - (block))");
    assert_eq!(stmt("လုပ်ဆောင်ချက် ပရိုဂရမ်ပိတ် ကို လုပ်ဆောင်ပါ။"), "(call ပရိုဂရမ်ပိတ်)");
    let args = "လုပ်ဆောင်ချက် ထုတ်ပြ ကို အသက်၊ အမည် တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(stmt(args), "(call ထုတ်ပြ အသက် အမည်)");
}

#[test]
fn statement_spans_cover_the_full_stop()
{
    let text = "က ကို ပြသပါ။ ကိန်းရှင် ခ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။";
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
    let const_loop = "ကိန်းသေ က သည် ကိန်းပြည့် ၀ ဖြစ်ပြီး က သည် ၁ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို \
                      လုပ်ဆောင်ပါ။ လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(const_loop)[0], Code::ConstLoop);
    let late = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပြီး က သည် ၁ အောက် ငယ်နေသရွေ့ အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
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
    assert_eq!(codes("\"x\" ဟု ထားပါ။"), [Code::MissingOperand]);
    assert_eq!(codes("၁ မှ ၁၀ အထိ ၁ + ၁ အနေဖြင့် ရေတွက်၍ က ကို ပြသပါ။"), [Code::ExpectedToken]);
}

#[test]
fn each_loops()
{
    let cells = "l ၏ အကွက်တိုင်း ကို x အနေဖြင့် ယူ၍ x ကို ပြသပါ။";
    assert_eq!(stmt(cells), "(each_cell x l (block (print x)))");
    let chars = "t ၏ စာလုံးတိုင်း ကို c အနေဖြင့် ယူ၍ c ကို ပြသပါ။";
    assert_eq!(stmt(chars), "(each_char c t (block (print c)))");
}

#[test]
fn counted_variable_joined_to_its_particle_gets_a_hint()
{
    let out = parse_text("၁ မှ ၁၀ အထိ ကိန်းအနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။");
    assert_eq!(out.diagnostics.len(), 1);
    let help = out.diagnostics[0].help.clone().unwrap_or_default();
    assert!(help.contains("ကိန်း အနေဖြင့်"), "{help}");
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

#[test]
fn pointer_declarations_and_places()
{
    assert_eq!(stmt("ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var p *isize)");
    assert_eq!(stmt("ကိန်းရှင် pp သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var pp **isize)");
    assert_eq!(stmt("ကိန်းရှင် b သည် ၈ ဘစ် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var b *i8)");
    assert_eq!(stmt("ကိန်းရှင် q သည် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var q ptr)");
    assert_eq!(stmt("ကိန်းရှင် qq သည် လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var qq *ptr)");
    assert_eq!(
        stmt("ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။"),
        "(var p *isize (addr x))"
    );
    assert_eq!(
        stmt("ကိန်းရှင် p သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။"),
        "(var p *isize (addr x))"
    );
    assert_eq!(stmt("p ၏ တန်ဖိုး ကို ၅ ဟု ထားပါ။"), "(set (deref p) 5)");
    assert_eq!(stmt("*p ကို *p + ၁ ဟု ထားပါ။"), "(set (deref p) (+ (deref p) 1))");
    assert_eq!(stmt("၅ ကို ၆ ဟု ထားပါ။"), "(set 5 6)"); // the checker rejects it, not the parser
}

#[test]
fn pointer_kinds_keep_the_older_spelling()
{
    assert_eq!(stmt("ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"), "(var p *isize)");
    assert_eq!(
        stmt("ကိန်းရှင် pp သည် ကိန်းပြည့် ၏ လိပ်စာ ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"),
        "(var pp **isize)"
    );
    assert_eq!(
        stmt("ကိန်းရှင် p သည် ကိန်းပြည့် ၏ လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် x ၏ လိပ်စာ ဖြစ်ပါစေ။"),
        "(var p *isize (addr x))"
    );
}

#[test]
fn borrowed_and_raw_kinds()
{
    let decl = |kind: &str| stmt(&format!("ကိန်းရှင် q သည် {kind} တစ်ခု ဖြစ်ပါစေ။"));
    assert_eq!(decl("ငှားထားသော ကိန်းပြည့် လိပ်စာ"), "(var q &*isize)");
    assert_eq!(decl("ငှားထားသော ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ"), "(var q &**isize)");
    assert_eq!(decl("ကိန်းပြည့် လိပ်စာအကြမ်း"), "(var q ~isize)");
    assert_eq!(decl("ကိန်းပြည့် လိပ်စာအကြမ်း ၏ လိပ်စာ"), "(var q *~isize)");
    assert_eq!(decl("ကိန်းပြည့် ၏ လိပ်စာအကြမ်း"), "(var q ~isize)");
    assert_eq!(decl("လိပ်စာအကြမ်း"), "(var q rawptr)");
    assert_eq!(decl("ငှားထားသော လိပ်စာ"), "(var q &ptr)");
    let list = "ငှားထားသော အကွက် ၃ ခုရှိသော ကိန်းပြည့် စာရင်း လိပ်စာ";
    assert_eq!(decl(list), "(var q &*[3]isize)");
}

#[test]
fn misplaced_borrow_markers_parse()
{
    let decl = |kind: &str| stmt(&format!("ကိန်းရှင် q သည် {kind} တစ်ခု ဖြစ်ပါစေ။"));
    assert_eq!(decl("ငှားထားသော ငှားထားသော ကိန်းပြည့် လိပ်စာ"), "(var q &&*isize)");
    assert_eq!(decl("ငှားထားသော ကိန်းပြည့် လိပ်စာအကြမ်း"), "(var q &~isize)");
    assert_eq!(decl("ငှားထားသော ကိန်းပြည့်"), "(var q &isize)");
}

#[test]
fn borrowed_kinds_in_function_headers()
{
    let head = "လုပ်ဆောင်ချက် f သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ က နှင့် ကိန်းပြည့် လိပ်စာအကြမ်း ခ တို့ကို \
                လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ငှားထားသော ကိန်းပြည့် လိပ်စာ ရလဒ် ကို ပြန်ပေးမည်။ \
                လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(stmt(head), "(fn f [&*isize က, ~isize ခ] &*isize (block))");
}

/// A parameter the function can change, and the mark at the call.
#[test]
fn edit_references_parse_at_both_ends()
{
    let decl = stmt(
        "လုပ်ဆောင်ချက် တိုး သည်
လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သော ကိန်းပြည့် န ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။",
    );
    assert!(decl.contains("edit isize န"), "{decl}");
    let call = stmt("လုပ်ဆောင်ချက် တိုး ကို က (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)၊ ခ တို့ဖြင့် လုပ်ဆောင်ပါ။");
    assert_eq!(call, "(call တိုး (edit က) ခ)");
    let sign = stmt("တိုး(ပ ၏ အသက် (လုပ်ဆောင်ချက်မှ ပြောင်းလဲနိုင်သည်)) ကို ပြသပါ။");
    assert!(sign.contains("(call တိုး (edit "), "{sign}");
}
