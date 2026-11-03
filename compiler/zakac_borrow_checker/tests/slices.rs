mod common;

use common::lines;
use common::main_forms;
use common::ok;
use common::run;
use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::Module;

/// `ပ` owns a heap block of three integers.
const BLOCK: &str = "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
မင်မိုရီတွင် ကိန်းပြည့် ၃ ခုစာ နေရာယူ၍ လိပ်စာကို ပ တွင် သိမ်းထားမည်။
";

#[test]
fn a_slice_of_a_block_is_a_loan_on_it()
{
    let used_after = format!(
        "{BLOCK}ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် \
         ဖြစ်ပါစေ။
လိပ်စာ ပ ရှိ မင်မိုရီကို ပြန်လည်စွန့်လွှတ်မည်။
s ကို ပြသပါ။
"
    );
    assert_eq!(lines(&used_after), [(Code::InvalidatedLoan, 4)]);
    ok(&format!(
        "{BLOCK}ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် \
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
        "{BLOCK}ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ပ ၏ အကွက် ၀ မှစ၍ ၃ ကွက် \
         ဖြစ်ပါစေ။
ကိန်းရှင် t သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် s ၏ အကွက် ၁ မှစ၍ ၁ ကွက် ဖြစ်ပါစေ။
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
ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂] ဖြစ်ပါစေ။
l ၏ တန်ဖိုးအငှား ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီး။
";
    // A plain list is dropped before the return, as text is, so the loan
    // also ends there; a fixed list in the frame reports the return alone.
    assert_eq!(
        lines(local),
        [(Code::LocalBorrowReturned, 4), (Code::InvalidatedLoan, 4)]
    );
    let fixed = local.replace("ကိန်းပြည့် စာရင်း တစ်ခု", "ကိန်းပြည့် စာရင်းသေ တစ်ခု");
    assert_eq!(lines(&fixed), [(Code::LocalBorrowReturned, 4)]);
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
    let text = "ကိန်းရှင် g သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်ပါစေ။
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
ကိန်းရှင် န သည် လက္ခဏာမဲ့ ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ၀ ဖြစ်ကာ န သည် ကိန်းများ ၏ အကွက်အရေအတွက် အောက် ငယ်နေသရွေ့ \
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
const KNOWN: &str = "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် s သည် ကိန်းပြည့် စာရင်း တစ်စိတ်တစ်ပိုင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် l ၏ အကွက် ၁ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
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

/// A write into a plain list can move its cells to a block of its own, so it
/// ends each loan into the list; a heap list keeps its cells in place.
#[test]
fn a_write_into_a_plain_list_ends_its_loans()
{
    let text = "ကိန်းရှင် l သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် v သည် l ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
l ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။
v ကို ပြသပါ။";
    assert_eq!(lines(text), [(Code::InvalidatedLoan, 3)]);
    let heap = text.replace(
        "[၁၊ ၂၊ ၃] ဖြစ်ပါစေ",
        "ကိန်းပြည့် စာရင်းရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ",
    );
    ok(&heap);
}

/// An add or a remove can move the cells of a plain or a heap list, so each
/// ends the loans into the list.
#[test]
fn adding_or_removing_a_cell_ends_the_loans_into_the_list()
{
    for edit in ["l ထဲသို့ ၉ ကို ထည့်ပါ။", "l ၏ နောက်ဆုံးအကွက်ကို ဖျက်ပါ။"]
    {
        let text = format!(
            "ကိန်းရှင် l သည် ကိန်းပြည့် စာရင်းရှင် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် v သည် l ၏ အကွက် ၀ မှစ၍ ၂ ကွက် ဖြစ်ပါစေ။
{edit}
v ကို ပြသပါ။"
        );
        assert_eq!(lines(&text), [(Code::InvalidatedLoan, 3)], "{edit}");
        ok(&text.replace("v ကို ပြသပါ။", "l ကို ပြသပါ။"));
    }
}

/// A character write or an add can move the u8s of text, so each ends the
/// views of it.
#[test]
fn a_text_edit_ends_its_views()
{
    for edit in ["t ထဲသို့ \"z\" ကို ထည့်ပါ။", "t ၏ စာလုံး ၀ ကို 'x' ဟု ထားပါ။"]
    {
        let text = format!(
            "ကိန်းရှင် t သည် \"ab\" ဖြစ်ပါစေ။
ကိန်းရှင် v သည် t ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
{edit}
v ကို ပြသပါ။"
        );
        assert_eq!(lines(&text), [(Code::InvalidatedLoan, 3)], "{edit}");
        ok(&text.replace("v ကို ပြသပါ။", "t ကို ပြသပါ။"));
    }
}

/// A pointer to a cell of a plain list points into the block of the list,
/// so a write into the list, or a new value for it, ends the loan.
#[test]
fn a_pointer_to_a_cell_of_a_plain_list_follows_its_block()
{
    let head = "ကိန်းရှင် က သည် [၁၊ ၂၊ ၃] ဖြစ်ပါစေ။
ကိန်းရှင် ပ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ၏ အကွက် ၁ ၏ လိပ်စာ ဖြစ်ပါစေ။
";
    for change in ["က ၏ အကွက် ၀ ကို ၉ ဟု ထားပါ။", "က ကို [၄၊ ၅] ဟု ထားပါ။"]
    {
        let text = format!("{head}{change}\nပ ၏ တန်ဖိုး ကို ပြသပါ။");
        assert_eq!(lines(&text), [(Code::InvalidatedLoan, 3)], "{change}");
    }
}
