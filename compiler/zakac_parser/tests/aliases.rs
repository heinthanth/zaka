mod common;

use std::path::Path;
use std::sync::Arc;

use common::codes;
use common::stmt;
use zakac_ast::s_expr::ToSexpr;
use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_parser::load::load as load_root;
use zakac_span::file_loader::MemFileLoader;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The declarations the programs below share.
const DECLS: &str = "ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဒေတာအမျိုးအစား အရောင် ဆိုသည်မှာ နီ၊ ဝါ တို့ ဖြစ်သည်။
ဒေတာအမျိုးအစား လူ ကို ပုဂ္ဂိုလ် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား ပုဂ္ဂိုလ် စာရင်း ကို လူစု ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား အရောင် ကို မီးရောင် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
";

/// Load `text` as one file.
fn load(text: &str) -> Loaded
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    Loaded::single(&file, &LexerOptions::default())
}

/// The trees of the start block of [`DECLS`] and `lines`, loaded with no
/// diagnostics.
fn start_block(lines: &str) -> String
{
    let text =
        format!("{DECLS}\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{lines}\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n");
    let loaded = load(&text);
    let msgs: Vec<&str> =
        loaded.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(msgs.is_empty(), "{lines}: {msgs:?}");
    loaded.program.stmts.last().expect("a start block").to_sexpr()
}

/// The diagnostic codes of a loaded program.
fn load_codes(text: &str) -> Vec<Code>
{
    load(text).diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn an_alias_reads_its_kind_and_its_name()
{
    let tree = stmt("ဒေတာအမျိုးအစား ၃၂ ဘစ် ကိန်းပြည့် ကို အမှတ် ဟုလည်း မှတ်ယူနိုင်ပါသည်။");
    assert!(tree.starts_with("(alias အမှတ် "), "{tree}");
    let spaced = stmt("ဒေတာအမျိုးအစား ကိန်းပြည့် ကို အမှတ် ဟု လည်း မှတ်ယူနိုင်ပါသည်။");
    assert!(spaced.starts_with("(alias အမှတ် "), "{spaced}");
}

#[test]
fn a_function_kind_needs_no_opener()
{
    let text = "ကိန်းပြည့် ကို လက်ခံသော လုပ်ဆောင်ချက် အမျိုးအစား ကို ပြချက် ဟုလည်း မှတ်ယူနိုင်ပါသည်။";
    assert!(stmt(text).starts_with("(alias ပြချက် "));
}

#[test]
fn another_kind_with_no_opener_is_reported()
{
    let text = "ကိန်းပြည့် ကို အမှတ် ဟုလည်း မှတ်ယူနိုင်ပါသည်။";
    assert_eq!(codes(text), [Code::ExpectedToken]);
}

#[test]
fn an_alias_inside_a_block_is_reported()
{
    let text = "ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ဒေတာအမျိုးအစား ကိန်းပြည့် ကို အမှတ် ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    assert_eq!(codes(text), [Code::NestedAlias]);
}

#[test]
fn a_kind_through_an_alias_is_the_kind_it_names()
{
    let pairs = [
        ("ကိန်းရှင် က သည် ပုဂ္ဂိုလ် တစ်ယောက် ဖြစ်ပါစေ။", "ကိန်းရှင် က သည် လူ တစ်ယောက် ဖြစ်ပါစေ။"),
        ("ကိန်းရှင် က သည် လူစု တစ်ခု ဖြစ်ပါစေ။", "ကိန်းရှင် က သည် လူ စာရင်း တစ်ခု ဖြစ်ပါစေ။"),
        ("ကိန်းရှင် က သည် လူစု ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။", "ကိန်းရှင် က သည် လူ စာရင်း ၏ လိပ်စာ တစ်ခု ဖြစ်ပါစေ။"),
        ("ပုဂ္ဂိုလ် ၏ အရွယ်အစား ကို ပြသပါ။", "လူ ၏ အရွယ်အစား ကို ပြသပါ။"),
    ];
    for (aliased, plain) in pairs
    {
        assert_eq!(start_block(aliased), start_block(plain), "{aliased}");
    }
}

#[test]
fn a_struct_literal_and_a_member_take_the_name_an_alias_stands_for()
{
    let pairs = [
        ("ပုဂ္ဂိုလ်(အသက်: ၁) ကို ပြသပါ။", "လူ(အသက်: ၁) ကို ပြသပါ။"),
        ("အသက် ၁ ရှိသော ပုဂ္ဂိုလ်တစ်ယောက် ကို ပြသပါ။", "အသက် ၁ ရှိသော လူတစ်ယောက် ကို ပြသပါ။"),
        ("မီးရောင်များထဲမှ ဝါ ကို ပြသပါ။", "အရောင်များထဲမှ ဝါ ကို ပြသပါ။"),
    ];
    for (aliased, plain) in pairs
    {
        assert_eq!(start_block(aliased), start_block(plain), "{aliased}");
    }
}

#[test]
fn an_alias_that_leads_back_to_itself_is_reported_once_for_each_alias()
{
    let text = "ဒေတာအမျိုးအစား ခ ကို က ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား က ကို ခ ဟုလည်း မှတ်ယူနိုင်ပါသည်။
ဒေတာအမျိုးအစား ဂ ကို ဂ ဟုလည်း မှတ်ယူနိုင်ပါသည်။";
    assert_eq!(load_codes(text), [Code::AliasLoop, Code::AliasLoop, Code::AliasLoop]);
}

#[test]
fn an_alias_takes_no_name_a_kind_has()
{
    let text = format!("{DECLS}ဒေတာအမျိုးအစား ကိန်းပြည့် ကို လူ ဟုလည်း မှတ်ယူနိုင်ပါသည်။\n");
    assert_eq!(load_codes(&text), [Code::DuplicateStruct]);
    let twice = format!("{DECLS}ဒေတာအမျိုးအစား ကိန်းပြည့် ကို ပုဂ္ဂိုလ် ဟုလည်း မှတ်ယူနိုင်ပါသည်။\n");
    assert_eq!(load_codes(&twice), [Code::DuplicateStruct]);
}

#[test]
fn an_exported_alias_starts_a_kind_in_the_file_that_imports_it()
{
    let lib = "အများသုံး ဒေတာအမျိုးအစား နေရာ တွင် ကိန်းပြည့် x တို့ ပါဝင်သည်။
အများသုံး ဒေတာအမျိုးအစား နေရာ ကို အမှတ် ဟုလည်း မှတ်ယူနိုင်ပါသည်။";
    let main = "\"lib.zaka\" မှ အမှတ် ကို ထည့်သွင်းပါ။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် က သည် အမှတ်(x: ၁) ဖြစ်ပါစေ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။";
    let mut files = MemFileLoader::new();
    files.insert("lib.zaka", lib);
    files.insert("main.zaka", main);
    let mut sm = SourceMap::with_loader(Arc::new(files));
    let root = sm.load_file(Path::new("main.zaka")).unwrap();
    let loaded = load_root(&root, &mut sm, &LexerOptions::default()).unwrap();
    let msgs: Vec<&str> =
        loaded.diagnostics.iter().map(|d| d.message.as_str()).collect();
    assert!(msgs.is_empty(), "{msgs:?}");
    let start = loaded.program.stmts.last().expect("a start block").to_sexpr();
    assert!(start.contains("နေရာ") && !start.contains("အမှတ်"), "{start}");
}
