use zakac_diagnostics::Code;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::structs::parse_sig;

use crate::common::check;
use crate::common::codes;
use crate::common::kind_at;
use crate::common::ok;

/// `လူ` with a padded field, and `အဆစ်`, which points at itself.
const STRUCTS: &str = "လူ ဟူသော ဒေတာအစုအဝေးတွင် အဆင့် ဟူသော ၈ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် အသက် ဟူသော \
                       ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
အဆစ် ဟူသော ဒေတာအစုအဝေးတွင် တန်ဖိုးရင်း ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် လိပ်စာ တစ်ခု တို့ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ထိပ် သည် အဆစ် တစ်ခု ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

/// The kind of a struct of [`STRUCTS`].
fn struct_kind(name: &str) -> DataKind { check(STRUCTS).out.structs[name] }

#[test]
fn fields_take_their_kind()
{
    let checked =
        check(&lines(&[STRUCTS, "ကျော် ၏ အသက် ကို ပြသမည်။", "ထိပ် ၏ နောက် ကို ပြသမည်။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ကျော် ၏ အသက်"), "isize");
    assert_eq!(kind_at(&checked, "ထိပ် ၏ နောက်"), "အဆစ်*");
    ok(&lines(&[STRUCTS, "ထိပ် ၏ နောက် ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို ၁ ဟု ထားမည်။"]));
}

#[test]
fn fields_sit_in_c_order()
{
    let s = struct_kind("လူ").as_struct().unwrap();
    assert_eq!((s.offset(0, 64), s.offset(1, 64)), (0, 8));
    assert_eq!((s.size(64), s.align(64)), (Some(16), 8));
    assert_eq!((s.offset(1, 32), s.size(32)), (4, Some(8)));
}

#[test]
fn a_struct_reads_back_from_its_signature()
{
    let kind = struct_kind("အဆစ်");
    let sig = kind.sig_name();
    assert_eq!(sig, "အဆစ်{တန်ဖိုးရင်း:isize,နောက်:အဆစ်*}");
    assert_eq!(DataKind::parse(&sig), Some(kind));
    assert_eq!(DataKind::parse(&format!("{sig}*")), Some(kind.pointer_to()));
    assert!(parse_sig("အဆစ်{တန်ဖိုးရင်း:nothing}").is_none());
}

/// A shared kind can hold an optional of itself, at once or through another
/// kind: the optional is made from the kinds of its sides, never read again
/// from its text, which would make the shared kind afresh without end.
#[test]
fn a_shared_kind_holds_an_optional_of_itself()
{
    let node = "အဆစ် ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် တန် ဟူသော ကိန်းပြည့် တစ်ခု နှင့် နောက် ဟူသော အဆစ် \
                ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု တို့ပါဝင်သည်။";
    let pair = "က ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ဘေး ဟူသော ခ ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု \
                ပါဝင်သည်။
ခ ဟူသော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် ပြန် ဟူသော က ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ပါဝင်သည်။";
    let uses = "ကိန်းရှင် ဒုတိယ သည် အဆစ်(တန်: ၂၊ နောက်: အလွတ်) ဖြစ်ပါစေ။
ကိန်းရှင် ပထမ သည် အဆစ်(တန်: ၁၊ နောက်: ဒုတိယ) ဖြစ်ပါစေ။
ပထမ ၏ နောက် ထဲမှ ဒေတာ ကို ပြသမည်။
ကိန်းရှင် ပ သည် က(ဘေး: ခ(ပြန်: အလွတ်)) ဖြစ်ပါစေ။";
    let checked = check(&lines(&[node, pair, uses]));
    assert!(checked.out.diagnostics.is_empty(), "{:?}", checked.out.diagnostics);
    assert_eq!(kind_at(&checked, "ပထမ ၏ နောက်"), "optional အဆစ်@");
}

#[test]
fn structs_of_one_kind_compare_each_field()
{
    ok(&lines(&[STRUCTS, "ကျော် == ကျော် ကို ပြသမည်။", "ထိပ် != ထိပ် ကို ပြသမည်။"]));
    let cells = "ကိန်းရှင် အုပ်စု သည် [ကျော်၊ ကျော်] ဖြစ်ပါစေ။";
    ok(&lines(&[STRUCTS, cells, "အုပ်စု == အုပ်စု ကို ပြသမည်။"]));
}

#[test]
fn struct_mistakes_have_their_codes()
{
    let decl = |text: &str| codes(&format!("လူ ဟူသော ဒေတာအစုအဝေးတွင် {text}ပါဝင်သည်။"));
    assert_eq!(
        decl("က ဟူသော ကိန်းပြည့် တစ်ခု နှင့် က ဟူသော ကိန်းပြည့် တစ်ခု တို့"),
        [Code::DuplicateField]
    );
    assert_eq!(decl("က ဟူသော လူ တစ်ခု "), [Code::StructHoldsItself]);
    assert_eq!(decl("က ဟူသော ကား တစ်ခု "), [Code::UnknownKindName]);
    assert_eq!(decl("လူ ဟူသော လူ လိပ်စာ တစ်ခု "), [Code::StructNameClash]);
    let used = |text: &str| codes(&lines(&[STRUCTS, text]));
    assert_eq!(used("ကျော် ၏ ဈေး ကို ပြသမည်။"), [Code::UnknownField]);
    assert_eq!(used("ကျော် ၏ အသက် ၏ အသက် ကို ပြသမည်။"), [Code::NotAStruct]);
    assert_eq!(used("ကိန်းရှင် လူ သည် ကိန်းပြည့် တစ်ခု ဖြစ်ပါစေ။"), [Code::StructNameClash]);
    assert_eq!(used("ကျော် < ကျော် ကို ပြသမည်။"), [Code::OperandKind]);
    assert_eq!(used("ကျော် == ထိပ် ကို ပြသမည်။"), [Code::IncompatibleOperands]);
    assert_eq!(used("ကျော် == ၁ ကို ပြသမည်။"), [Code::IncompatibleOperands]);
    let twice = "လူ ဟူသော ဒေတာအစုအဝေးတွင် က ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။";
    assert_eq!(codes(&lines(&[STRUCTS, twice])), [Code::DuplicateStruct]);
    let result = "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ တစ်ခု ကို ပြန်ပေးမည်။\nကိန်းရှင် ပ သည် လူ \
                  တစ်ခု ဖြစ်ပါစေ။\nပ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    ok(&lines(&[STRUCTS, result]));
}
