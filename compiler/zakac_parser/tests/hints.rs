use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::stmts;

/// The trees of a program that must parse with no diagnostics, one line each.
fn trees(text: &str) -> String
{
    let list = stmts(text);
    list.iter().map(ToSexpr::to_sexpr).collect::<Vec<_>>().join("\n")
}

/// The tree of the kind of a variable declared with `kind`, after `decls`.
fn kind_tree(decls: &str, kind: &str) -> String
{
    let text = format!("{decls}ကိန်းရှင် တ သည် {kind} တစ်ခု ဖြစ်ပါစေ။\n");
    let all = trees(&text);
    all.lines().last().unwrap_or_default().to_string()
}

/// A pair whose hint puts its kinds in the order a reader speaks them: the
/// second kind first.
const PAIR: &str = "အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ \
                    တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု တို့ပါဝင်ပြီး \
                    မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်ကာ ရေးသားရာတွင် ခ ကို က ဖြင့် တွဲသော အတွဲဟု ရေးသားနိုင်သည်။\n";

/// The list and the table of the core read the same in their hints, in
/// the long form and in the notation form.
#[test]
fn core_kinds_read_alike_in_each_form()
{
    let list = kind_tree("", "ကိန်းပြည့် စာရင်း");
    assert_eq!(kind_tree("", "ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် စာရင်း"), list);
    assert_eq!(kind_tree("", "စာရင်း<ကိန်းပြည့်>"), list);
    let table = kind_tree("", "စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား");
    let long = kind_tree("", "စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် ဇယား");
    assert_eq!(long, table);
    assert_eq!(kind_tree("", "ဇယား<စာသား၊ ကိန်းပြည့်>"), table);
}

/// A hint can put the kind parameters in each order; a use gives them back
/// in the order of the declaration.
#[test]
fn a_hint_gives_the_kinds_in_declared_order()
{
    let hinted = kind_tree(PAIR, "စာသား ကို ကိန်းပြည့် ဖြင့် တွဲသော အတွဲ");
    assert!(hinted.contains("အတွဲ<isize text>"), "{hinted}");
    let long = kind_tree(PAIR, "ကိန်းပြည့် နှင့် စာသား တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ");
    assert_eq!(long, hinted);
}

/// A kind in a slot can be a hinted kind itself, and a hint level can follow
/// another level.
#[test]
fn hints_nest()
{
    let lists = kind_tree("", "ကိန်းပြည့် စာရင်း စာရင်း");
    let long = "ကိန်းပြည့် စာရင်း ကို အစားထိုးအသုံးပြုသည့် စာရင်း";
    assert_eq!(kind_tree("", long), lists);
    let table = kind_tree("", "စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် စာရင်း ဇယား စာရင်း");
    let long = "စာသား နှင့် ကိန်းပြည့် စာရင်း တို့ကို အစားထိုးအသုံးပြုသည့် ဇယား စာရင်း";
    assert_eq!(kind_tree("", long), table);
    let counted = kind_tree("", "အကွက် ၂ ခုရှိသော စာသား ဖြင့် ညွှန်းသော ကိန်းပြည့် ဇယား စာရင်း");
    assert!(counted.contains("(map text isize)[2]"), "{counted}");
    let pair = kind_tree(PAIR, "စာသား စာရင်း ကို ကိန်းပြည့် ဖြင့် တွဲသော အတွဲ");
    assert!(pair.contains("အတွဲ<isize text[]>"), "{pair}");
}

/// A hint that breaks a rule of its form is ZK0252.
#[test]
fn a_bad_hint_is_reported()
{
    let head = "က ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B \
                တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် x ဟူသော A တစ်ခု နှင့် y ဟူသော B တစ်ခု တို့ပါဝင်ပြီး \
                မျိုးပြပစ္စည်းမှာ ခု ဖြစ်ကာ ရေးသားရာတွင်";
    let bad = |hint: &str| codes(&format!("{head} {hint} ရေးသားနိုင်သည်။\n"));
    for hint in ["A B ကဟု", "A ဖြင့် B ခဟု", "A ဖြင့် ကဟု", "A ၊ B ကဟု"]
    {
        assert_eq!(bad(hint), vec![Code::BadHint], "{hint}");
    }
    assert!(bad("A ဖြင့် B ကဟု").is_empty());
    assert!(bad("A ဖြင့် B က ဟု").is_empty());
    assert!(bad("A ဖြင့် A ဖြင့် B ကဟု").is_empty());
}

/// A file that declares a kind named `စာရင်း` hides the list of the core.
#[test]
fn a_kind_of_the_file_hides_the_core_list()
{
    let own = "စာရင်း ဟူသော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။\n";
    let tree = kind_tree(own, "စာရင်း");
    assert!(tree.contains("စာရင်း"), "{tree}");
    assert!(!tree.contains("[]"), "{tree}");
}

const CONVERT: &str = "ပြောင်းနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A နှင့် \
                       ယေဘုယျဒေတာအမျိုးအစား B တို့ကိုအစားထိုးသုံးနိုင်၍ ရေးသားရာတွင် B အဖြစ် \
                       ပြောင်းနိုင်သောဟု ရေးသားနိုင်ပြီး ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ \
                       ပါဝင်ရမည်။\nပြောင်း ဟူသော လုပ်ဆောင်ချက် သည်\nက ဟူသော A တစ်ခု ကို လက်ခံပြီး B \
                       တစ်ခု ကို ပြန်ပေးမည်။\nဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n";

/// A function that needs the trait of [`CONVERT`], named as `trait`.
fn needs(trait_ref: &str) -> String
{
    format!(
        "{CONVERT}ပြ ဟူသော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်\nက ဟူသော \
         T တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။\nဤနေရာတွင် သတိပြုရန်မှာ\nT \
         တွင် {trait_ref} ဂုဏ်သတ္တိ ရှိသည်။\nသတိပြုစရာများ ပြီးဆုံးပါပြီး။\nပြောင်း(က) ကို \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
    )
}

/// A trait takes its kinds after the first through its hint or the long
/// form; the old `ကို သုံး၍` form is a plain parse error.
#[test]
fn a_trait_takes_its_kinds_through_its_hint()
{
    let hinted = trees(&needs("စာသား အဖြစ် ပြောင်းနိုင်သော"));
    assert!(hinted.contains("ပြ<T:ပြောင်းနိုင်သော<text>>"), "{hinted}");
    let long = trees(&needs("စာသား ကို အစားထိုးအသုံးပြုသည့် ပြောင်းနိုင်သော"));
    assert_eq!(long, hinted);
    assert!(!codes(&needs("စာသား ကို သုံး၍ ပြောင်းနိုင်သော")).is_empty());
    let first = CONVERT.replace("B အဖြစ်", "A အဖြစ် B");
    assert_eq!(codes(&first), vec![Code::BadHint]);
}

/// A trait and a struct declared in the sign form take a hint as the word
/// form does.
#[test]
fn the_sign_form_takes_a_hint()
{
    let sign = CONVERT.replacen(
        "ပြောင်းနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A နှင့် ယေဘုယျဒေတာအမျိုးအစား B \
         တို့ကိုအစားထိုးသုံးနိုင်၍",
        "ပြောင်းနိုင်သော<A၊ B> ဟူသည့် ဂုဏ်သတ္တိ သည်",
        1,
    );
    assert_ne!(sign, CONVERT);
    let word = trees(&needs("စာသား အဖြစ် ပြောင်းနိုင်သော"));
    let signed = trees(&needs("စာသား အဖြစ် ပြောင်းနိုင်သော").replace(CONVERT, &sign));
    assert_eq!(signed, word);
    let pair = "အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင် ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု တို့ပါဝင်ပြီး \
                မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်ကာ ရေးသားရာတွင် က နှင့် ခ အတွဲဟု ရေးသားနိုင်သည်။\n";
    let hinted = kind_tree(pair, "စာသား နှင့် ကိန်းပြည့် အတွဲ");
    assert!(hinted.contains("အတွဲ<text isize>"), "{hinted}");
}

/// A pair of two values of one kind, whose hint names its kind parameter
/// twice.
const TWIN: &str = "ညီအတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် ပထမ \
                    ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော က တစ်ခု တို့ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ တွဲ ဖြစ်ကာ \
                    ရေးသားရာတွင် က ကို က ဖြင့် တွဲသော ညီအတွဲဟု ရေးသားနိုင်သည်။\n";

/// A kind parameter that stands twice in a hint takes one kind, which a use
/// writes in each place; another kind in the second place is ZK0253.
#[test]
fn a_repeated_slot_takes_one_kind()
{
    let twin = kind_tree(TWIN, "ကိန်းပြည့် ကို ကိန်းပြည့် ဖြင့် တွဲသော ညီအတွဲ");
    assert!(twin.contains("ညီအတွဲ<isize>"), "{twin}");
    let long = kind_tree(TWIN, "ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် ညီအတွဲ");
    assert_eq!(long, twin);
    let mixed = format!("{TWIN}ကိန်းရှင် တ သည် ကိန်းပြည့် ကို စာသား ဖြင့် တွဲသော ညီအတွဲ တစ်ခု ဖြစ်ပါစေ။\n");
    assert_eq!(codes(&mixed), vec![Code::HintSlotTwoKinds]);
}
