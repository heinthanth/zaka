mod common;

use common::check;
use common::codes;
use common::kind_at;
use common::ok;
use zakac_diagnostics::Code;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::structs::parse_sig;

/// `လူ` with a padded field, and `အဆစ်`, which points at itself.
const STRUCTS: &str = "ဒေတာအမျိုးအစား လူ တွင် ၈ ဘစ် ကိန်းပြည့် အဆင့်၊ ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။
ဒေတာအမျိုးအစား အဆစ် တွင် ကိန်းပြည့် တန်ဖိုးရင်း၊ အဆစ် လိပ်စာ နောက် တို့ ပါဝင်သည်။
ကိန်းရှင် ကျော် သည် လူ ဖြစ်ပါစေ။
ကိန်းရှင် ထိပ် သည် အဆစ် ဖြစ်ပါစေ။
";

/// Join program lines.
fn lines(parts: &[&str]) -> String { parts.join("\n") }

/// The kind of a struct of [`STRUCTS`].
fn struct_kind(name: &str) -> DataKind { check(STRUCTS).out.structs[name] }

#[test]
fn fields_take_their_kind()
{
    let checked =
        check(&lines(&[STRUCTS, "ကျော် ၏ အသက် ကို ပြသပါ။", "ထိပ် ၏ နောက် ကို ပြသပါ။"]));
    assert!(checked.out.diagnostics.is_empty());
    assert_eq!(kind_at(&checked, "ကျော် ၏ အသက်"), "isize");
    assert_eq!(kind_at(&checked, "ထိပ် ၏ နောက်"), "*အဆစ်");
    ok(&lines(&[STRUCTS, "ထိပ် ၏ နောက် ၏ တန်ဖိုး ၏ တန်ဖိုးရင်း ကို ၁ ဟု ထားပါ။"]));
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
    assert_eq!(sig, "အဆစ်{တန်ဖိုးရင်း:isize,နောက်:*အဆစ်}");
    assert_eq!(DataKind::parse(&sig), Some(kind));
    assert_eq!(DataKind::parse(&format!("*{sig}")), Some(kind.pointer_to()));
    assert!(parse_sig("အဆစ်{တန်ဖိုးရင်း:nothing}").is_none());
}

#[test]
fn an_owner_field_makes_a_move_kind()
{
    assert!(!struct_kind("လူ").is_move_kind());
    assert!(struct_kind("အဆစ်").is_move_kind());
}

#[test]
fn struct_mistakes_have_their_codes()
{
    let decl = |text: &str| codes(&format!("ဒေတာအမျိုးအစား လူ တွင် {text} ပါဝင်သည်။"));
    assert_eq!(decl("ကိန်းပြည့် က၊ ကိန်းပြည့် က တို့"), [Code::DuplicateField]);
    assert_eq!(decl("လူ က တို့"), [Code::StructHoldsItself]);
    assert_eq!(decl("ကား က တို့"), [Code::UnknownKindName]);
    assert_eq!(decl("လူ လိပ်စာ လူ တို့"), [Code::StructNameClash]);
    assert_eq!(decl("ငှားထားသော ကိန်းပြည့် လိပ်စာ က တို့"), [Code::BorrowField]);
    let used = |text: &str| codes(&lines(&[STRUCTS, text]));
    assert_eq!(used("ကျော် ၏ ဈေး ကို ပြသပါ။"), [Code::UnknownField]);
    assert_eq!(used("ကျော် ၏ အသက် ၏ အသက် ကို ပြသပါ။"), [Code::NotAStruct]);
    assert_eq!(used("ကိန်းရှင် လူ သည် ကိန်းပြည့် ဖြစ်ပါစေ။"), [Code::StructNameClash]);
    assert_eq!(used("ကျော် == ကျော် ကို ပြသပါ။"), [Code::OperandKind]);
    let twice = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် က တို့ ပါဝင်သည်။";
    assert_eq!(codes(&lines(&[STRUCTS, twice])), [Code::DuplicateStruct]);
    let result =
        "လုပ်ဆောင်ချက် f သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ လူ ရလဒ် ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(codes(&lines(&[STRUCTS, result])), [Code::StructResult]);
}
