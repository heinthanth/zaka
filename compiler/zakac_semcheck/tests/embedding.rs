//! Structs based on other structs: the base is a struct (ZK0363), a name in
//! two parts at one depth is ZK0364, and a kind from outside is based on
//! nothing (ZK0263).

use zakac_diagnostics::Code;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_parser::parser::parse;
use zakac_span::filename::FileName;
use zakac_span::source_file::SourceFile;
use zakac_span::source_map::SourceMap;

use crate::common::codes_as_is;

/// The errors of bases and of names in two parts.
#[test]
fn bases_are_structs_and_names_pick_one_part()
{
    let text = r#"လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဝန်ထမ်း ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဆရာ ဟူသော ဒေတာအစုအဝေး သည် လူ နှင့် ဝန်ထမ်း တို့ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍
၎င်းတွင် ဘာသာ ဟူသော စာသား တစ်ခု ပါဝင်ပြီး မျိုးပြပစ္စည်းမှာ ယောက် ဖြစ်သည်။
ဂဏန်း ဟူသော ဒေတာအစုအဝေး သည် ကိန်းပြည့် ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်၍
၎င်းတွင် ဘာသာ ဟူသော စာသား တစ်ခု ပါဝင်သည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် ဆ သည် ဆရာ(လူ: လူ(နာမည်: "က")၊ ဝန်ထမ်း: ဝန်ထမ်း(နာမည်: "ခ")၊ ဘာသာ: "x") ဖြစ်ပါစေ။
ဆ ၏ နာမည် ကို ပြသမည်။
(ဆ ကို ဝန်ထမ်း အဖြစ် ပြောင်းလဲသော ရလဒ်) ၏ နာမည် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"#;
    let found = codes_as_is(text);
    assert!(found.contains(&Code::BaseNotAStruct), "{found:?}");
    assert!(found.contains(&Code::BaseNameTwice), "{found:?}");
}

/// Two marks of a head need `၊` between them (ZK0266), and a kind from
/// outside is based on no struct (ZK0263).
#[test]
fn marks_of_a_head_take_a_separator()
{
    let person = "လူ ဟူသော ဒေတာအစုအဝေးတွင် နာမည် ဟူသော စာသား တစ်ခု ပါဝင်သည်။\n";
    let glued = format!(
        "{person}က ဟူသော အများသုံးနိုင်သော မျှဝေနိုင်သော ဒေတာအစုအဝေးတွင် x ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။"
    );
    let lexed = lex(&file(&glued), &LexerOptions::default());
    let parsed = parse(&file(&glued), &lexed);
    let codes: Vec<Code> = parsed.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&Code::MarksNeedSeparator), "{codes:?}");
    let outside = format!(
        "{person}ခ ဟူသော ဒေတာအစုအဝေး သည် ပြင်ပမှ လာသော၊ လူ ကို အခြေခံသော ဒေတာအစုအဝေး ဖြစ်သည်။"
    );
    let lexed = lex(&file(&outside), &LexerOptions::default());
    let parsed = parse(&file(&outside), &lexed);
    let codes: Vec<Code> = parsed.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&Code::MarkClash), "{codes:?}");
}

/// A source file of a text.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The file.
fn file(text: &str) -> std::sync::Arc<SourceFile>
{
    let mut sm = SourceMap::new();
    sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string())
}
