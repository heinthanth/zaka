mod common;

use common::lines;
use common::main_forms;
use common::ok;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::Module;

/// `ပ` owns a heap block of three integers.
const BLOCK: &str = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
";

#[test]
fn a_slice_of_a_block_is_a_loan_on_it()
{
    let used_after = format!(
        "{BLOCK}ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် \
         ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
s ကို ပြသပါ။
"
    );
    assert_eq!(lines(&used_after), [(Code::InvalidatedLoan, 4)]);
    ok(&format!(
        "{BLOCK}ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် \
         ဖြစ်ပါစေ။
s ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ၏ အကွက် ၀ ကို ၅ ဟု ထားပါ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
"
    ));
}

#[test]
fn a_slice_of_a_slice_keeps_the_loan()
{
    let text = format!(
        "{BLOCK}ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် \
         ဖြစ်ပါစေ။
ကိန်းရှင် t သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် s ၏ အကွက် ၁ မှစ၍ ၁ ကွက် ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
t ကို ပြသပါ။
"
    );
    assert_eq!(lines(&text), [(Code::InvalidatedLoan, 5)]);
}

#[test]
fn a_slice_returned_from_a_function_follows_its_source()
{
    let local = "လုပ်ဆောင်ချက် f သည်
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
l ၏ တန်ဖိုးအငှား ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
";
    assert_eq!(lines(local), [(Code::LocalBorrowReturned, 4)]);
    ok("လုပ်ဆောင်ချက် f သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း s ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ရလဒ် ကို ပြန်ပေးမည်။
s ၏ အကွက် ၁ မှစ၍ ၁ ကွက် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
");
}

#[test]
fn a_function_cannot_keep_a_slice_in_a_global()
{
    let text = "ကိန်းရှင် g သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် f သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း s ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်မည်။
g ကို s ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
";
    assert_eq!(lines(text), [(Code::BorrowStored, 5)]);
}

/// The folded IR of a text that checks clean.
fn module(text: &str) -> Module
{
    let run = run(text);
    run.out.lowered.folded.expect("the text lowers")
}

/// The statements of the first function, as text.
fn func_forms(module: &Module) -> Vec<String>
{
    module.funcs[0].block.iter().map(ToSexpr::to_sexpr).collect()
}

/// A sum over a slice parameter, its counter tested against the count.
const SUM: &str = "လုပ်ဆောင်ချက် ပေါင်း သည်
ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ကိန်းများ ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
ကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ \
                   အောက်ပါတို့ကို လုပ်ဆောင်ပါ။
BODY
ပြီးလျှင် န ကို န + ၁ ဟု ထားပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
";

#[test]
fn a_counter_below_the_count_reads_a_slice_unchecked()
{
    let text = SUM.replace("BODY", "ရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။");
    ok(&text);
    let forms = func_forms(&module(&text)).join(" ");
    assert!(forms.contains("count checked"), "{forms}");
}

#[test]
fn a_counter_changed_after_the_test_proves_nothing()
{
    let changed = "န ကို န + ၁ ဟု ထားပါ။\nရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ အကွက် န) ဟု ထားပါ။";
    let forms = func_forms(&module(&SUM.replace("BODY", changed))).join(" ");
    assert!(forms.contains("count)") && !forms.contains("count checked"), "{forms}");
    let other = "ကိန်းများ ကို ကိန်းများ ၏ အကွက် ၁ မှစ၍ ၀ ကွက် ဟု ထားပါ။\nရလဒ် ကို ရလဒ် + (ကိန်းများ ၏ \
                 အကွက် န) ဟု ထားပါ။";
    let forms = func_forms(&module(&SUM.replace("BODY", other))).join(" ");
    assert!(!forms.contains("count checked"), "{forms}");
}

/// A list of three cells and a slice of its last two.
const KNOWN: &str = "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
";

#[test]
fn a_slice_of_a_known_count_checks_its_cells_at_compile_time()
{
    let inside = format!("{KNOWN}s ၏ အကွက် ၁ ကို ပြသပါ။");
    ok(&inside);
    let forms = main_forms(&module(&inside)).join(" ");
    assert!(forms.contains("count checked"), "{forms}");
    let past = format!("{KNOWN}s ၏ အကွက် ၂ ကို ပြသပါ။");
    assert_eq!(lines(&past), [(Code::OffsetPastEnd, 3)]);
}
