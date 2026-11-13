use zakac_ast::s_expr::ToSexpr;
use zakac_ast::stmt::Stmt;
use zakac_diagnostics::Code;

use crate::common::codes;
use crate::common::stmts;

/// The trees of a program that must parse with no diagnostics, one line each.
fn trees(text: &str) -> String
{
    let list = stmts(text);
    list.iter().map(ToSexpr::to_sexpr).collect::<Vec<_>>().join("\n")
}

const PAIR: &str =
    "အတွဲ<က၊ ခ> ဟူသော ဒေတာအစုအဝေးတွင် ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု တို့ပါဝင်သည်။\n";

/// A struct, a function and an alias take kind parameters after their name
/// in the notation form.
#[test]
fn declarations_take_kind_parameters_in_the_notation_form()
{
    assert!(trees(PAIR).contains("အတွဲ<က ခ>"));
    let f = "လုပ်ဆောင်ချက် ထပ်<T> သည်\nက ဟူသော T တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို \
             ပြန်ပေးမည်။\nက ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    assert!(trees(f).contains("ထပ်<T>"), "{}", trees(f));
    let bound = f.replace("ထပ်<T>", "ထပ်<T: စီနိုင်သော နှင့် ပြနိုင်သော>");
    assert!(trees(&bound).contains("ထပ်<T:စီနိုင်သော+ပြနိုင်သော>"), "{}", trees(&bound));
    let alias = "ဒေတာအမျိုးအစား အမည်ဇယား<T> ကို စာသား ဖြင့် ညွှန်းသော T ဇယား ဟုလည်း မှတ်ယူနိုင်ပါသည်။\n";
    assert!(trees(alias).contains("အမည်ဇယား<T>"), "{}", trees(alias));
}

/// The word form names each kind parameter after `ယေဘုယျဒေတာအမျိုးအစား`, then
/// closes the list with `ကိုအစားထိုးသုံးနိုင်၍ ၎င်း`.
#[test]
fn declarations_take_kind_parameters_in_the_word_form()
{
    let f = "လုပ်ဆောင်ချက် ထပ် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်\nက ဟူသော T တစ်ခု ကို \
             လက်ခံပြီး\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။\nက ကို \
             ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    assert!(trees(f).contains("ထပ်<T>"), "{}", trees(f));
    let s = "အတွဲ ဟူသော ဒေတာအစုအဝေးသည် ယေဘုယျဒေတာအမျိုးအစား က နှင့် ယေဘုယျဒေတာအမျိုးအစား ခ \
             တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် ပထမ ဟူသော က တစ်ခု နှင့် ဒုတိယ ဟူသော ခ တစ်ခု တို့ပါဝင်သည်။\n";
    assert!(trees(s).contains("အတွဲ<က ခ>"), "{}", trees(s));
    let alias = "ဒေတာအမျိုးအစား ပြန်တွဲ သည် ယေဘုယျဒေတာအမျိုးအစား K နှင့် ယေဘုယျဒေတာအမျိုးအစား V \
                 တို့ကိုအစားထိုးသုံးနိုင်၍ ၎င်းကို V နှင့် K တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ ဟုလည်း မှတ်ယူနိုင်ပါသည်။\n";
    let alias = trees(&format!("{PAIR}{alias}"));
    assert!(alias.contains("ပြန်တွဲ<K V>"), "{alias}");
}

/// A kind takes its kind arguments in the sign form, nested as written, or
/// in the word form before its name.
#[test]
fn kinds_take_kind_arguments()
{
    let decl = |kind: &str| format!("{PAIR}ကိန်းရှင် တ သည် {kind} တစ်ခု ဖြစ်ပါစေ။\n");
    let sign = trees(&decl("အတွဲ<စာသား၊ ကိန်းပြည့်>"));
    assert!(sign.contains("အတွဲ<text isize>"), "{sign}");
    let nested = trees(&decl("အတွဲ<အတွဲ<စာသား၊ ကိန်းပြည့်>၊ ကိန်းပြည့်>"));
    assert!(nested.contains("အတွဲ<အတွဲ<text isize> isize>"), "{nested}");
    let word = trees(&decl("စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် အတွဲ"));
    assert!(word.contains("အတွဲ<text isize>"), "{word}");
}

/// A call, a function value and a struct literal take kind arguments after
/// `::`, and the word forms of a call and a struct value take them before
/// the name.
#[test]
fn calls_take_kind_arguments()
{
    let sign = trees("f::<ကိန်းပြည့်>(၁) ကို ပြသမည်။\n");
    assert!(sign.contains("(call f<isize> 1)"), "{sign}");
    let value = trees("ကိန်းရှင် g သည် f::<ကိန်းပြည့်> ဖြစ်ပါစေ။\n");
    assert!(value.contains("(fn_addr f<isize>)"), "{value}");
    let lit =
        trees(&format!("{PAIR}အတွဲ::<စာသား၊ ကိန်းပြည့်>(ပထမ: \"က\"၊ ဒုတိယ: ၁) ကို ပြသမည်။\n"));
    assert!(lit.contains("(struct အတွဲ<text isize>"), "{lit}");
    let long = trees("ကိန်းပြည့် ကို အစားထိုးအသုံးပြုသည့် f ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်မည်။\n");
    assert!(long.contains("(call f<isize> 1)"), "{long}");
    let word = trees(&format!(
        "{PAIR}ပထမ အနေဖြင့် \"က\" နှင့် ဒုတိယ အနေဖြင့် ၁ ရှိသော စာသား နှင့် ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြုသည့် \
         အတွဲ တစ်ခု ကို ပြသမည်။\n"
    ));
    assert!(word.contains("(struct အတွဲ<text isize>"), "{word}");
    let first = trees("ကိန်းပြည့် ကို အစားထိုးအသုံးပြု၍ ၁ ကို သုံး၍ f မည်။\n");
    assert!(first.contains("f<isize>"), "{first}");
}

/// The kind arguments in the word form close with the word of their count.
#[test]
fn word_kind_arguments_close_with_the_word_of_their_count()
{
    let bad = "ကိန်းပြည့် တို့ကို အစားထိုးအသုံးပြု၍ ၁ ကို သုံး၍ f မည်။\n";
    assert!(!codes(bad).is_empty(), "{bad}");
}

/// A trait and its function heads, in the word form and the notation form,
/// with the traits it needs in a notes block.
#[test]
fn traits_name_their_function_heads()
{
    let word = "စီနိုင်သော ဟူသည့် ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် \
                အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။\nလုပ်ဆောင်ချက် ငယ်သလား သည်\nက ဟူသော A တစ်ခု ကို \
                လည်းကောင်း ခ ဟူသော A တစ်ခု ကို လည်းကောင်း\nလက်ခံပြီး မှန်မှားတန်ဖိုး တစ်ခု ကို \
                ပြန်ပေးမည်။\nဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n";
    let tree = trees(word);
    assert!(tree.starts_with("(trait စီနိုင်သော<A> (fn ငယ်သလား"), "{tree}");
    let sign = "ပြောင်းနိုင်သော<A၊ B> ဟူသည့် ဂုဏ်သတ္တိ တွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။\nဤနေရာတွင် \
                သတိပြုရန်မှာ\nA တွင် စီနိုင်သော ဂုဏ်သတ္တိ နှင့် ပြနိုင်သော ဂုဏ်သတ္တိ တို့ ရှိသည်။\nသတိပြုစရာများ \
                ပြီးဆုံးပါပြီး။\nလုပ်ဆောင်ချက် ပြောင်း သည်\nက ဟူသော A တစ်ခု ကို လက်ခံပြီး B တစ်ခု ကို \
                ပြန်ပေးမည်။\nဂုဏ်သတ္တိတွင် ပါသော လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n";
    let tree = trees(sign);
    assert!(tree.starts_with("(trait ပြောင်းနိုင်သော<A:စီနိုင်သော+ပြနိုင်သော B>"), "{tree}");
}

/// A trait block names its kind and its trait; its functions become
/// functions of the file, after the block.
#[test]
fn a_trait_block_moves_its_functions_out()
{
    let block = "ကိန်းပြည့် တွင် စီနိုင်သော ဂုဏ်သတ္တိရှိစေရန် အောက်ပါလုပ်ဆောင်ချက်များ ကို \
                 ထည့်သွင်းမည်။\nလုပ်ဆောင်ချက် ငယ်သလား သည်\nက ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း ခ ဟူသော \
                 ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း\nလက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို \
                 ပြန်ပေးမည်။\nက < ခ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nထည့်သွင်းမည့် \
                 လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n";
    let tree = trees(block);
    let lines: Vec<&str> = tree.lines().collect();
    assert_eq!(lines.len(), 2, "{tree}");
    assert!(lines[0].starts_with("(impl isize စီနိုင်သော [ငယ်သလား]"), "{tree}");
    assert!(lines[1].starts_with("(fn ငယ်သလား"), "{tree}");
}

/// A notes block names local kinds, which stand for their kinds in the
/// whole function, and the traits of its kind parameters; a note on a name
/// that is no kind parameter is ZK0248.
#[test]
fn a_notes_block_names_local_kinds_and_traits()
{
    let f = "လုပ်ဆောင်ချက် လှည့်<A> သည်\nတွဲ ဟူသော C တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ C တစ်ခု ကို \
             ပြန်ပေးမည်။\nဤနေရာတွင် သတိပြုရန်မှာ\nC သည် A စာရင်း တစ်ခု ဖြစ်သည်။\nA တွင် စီနိုင်သော ဂုဏ်သတ္တိ \
             ရှိသည်။\nသတိပြုစရာများ ပြီးဆုံးပါပြီး။\nတွဲ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";
    let tree = trees(f);
    assert!(tree.contains("လှည့်<A:စီနိုင်သော> [A[] တွဲ] A[]"), "{tree}");
    let other = f.replace("A တွင်", "X တွင်");
    assert_eq!(codes(&other), [Code::NoteNamesNoParam]);
}

/// The prelude marks the traits the compiler gives, in the slot of
/// `ပြင်ပမှလာသော`: a trait.
#[test]
fn the_compiler_mark_stands_where_an_external_mark_stands()
{
    let t = "ပြနိုင်သော ဟူသည့် အများသုံး ကွန်ပိုင်လာထဲတွင် တစ်ပါတည်းပါဝင်သော ဂုဏ်သတ္တိ သည် ယေဘုယျဒေတာအမျိုးအစား \
             A ကိုအစားထိုးသုံးနိုင်၍ ၎င်းတွင် အောက်ပါလုပ်ဆောင်ချက်များ ပါဝင်ရမည်။\nဂုဏ်သတ္တိတွင် ပါသော \
             လုပ်ဆောင်ချက်များ ပြီးဆုံးပါပြီး။\n";
    let list = stmts(t);
    assert!(
        matches!(list.as_slice(), [Stmt::Trait(d)] if d.builtin && d.exported),
        "{list:?}"
    );
}
