use zakac_diagnostics::Code;
use zakac_diagnostics::Level;
use zakac_lexer::lexer::LexerOptions;
use zakac_lexer::lexer::lex;
use zakac_naturalness::NaturalnessOptions;
use zakac_naturalness::check;
use zakac_parser::parser::parse;
use zakac_span::filename::FileName;
use zakac_span::source_map::SourceMap;

/// The particle codes of `text`; panics on lexer or parser diagnostics.
fn warns_with(text: &str, options: &NaturalnessOptions) -> Vec<Code>
{
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let parsed = parse(&file, &lexed);
    let mut front = lexed.diagnostics;
    front.extend(parsed.diagnostics);
    let msgs: Vec<&str> = front.iter().map(|d| d.message.as_str()).collect();
    assert!(front.is_empty(), "front-end errors in {text}: {msgs:?}");
    check(&parsed.program, options).iter().map(|d| d.code).collect()
}

/// The particle codes of `text` with the default options.
fn warns(text: &str) -> Vec<Code> { warns_with(text, &NaturalnessOptions::default()) }

/// Assert that `text` has no particle warnings.
fn clean(text: &str)
{
    assert_eq!(warns(text), [], "{text}");
}

#[test]
fn negative_before_or_wants_pa()
{
    let text = "(က သည် ၁ အောက် မငယ်သည် သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသပါ။";
    assert_eq!(warns(text), [Code::NegativeBeforeOr]);
    clean("(က သည် ၁ အောက် မငယ်ပါ သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသပါ။");
    clean("(က သည် ၁ အောက် ငယ်သည် သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသပါ။");
    assert_eq!(warns("(မမှန်သည် သို့မဟုတ် မှန်သည်) ဟု ပြသပါ။"), [Code::NegativeBeforeOr]);
    clean("(က မဟုတ် သို့မဟုတ် ခ) ကို ပြသပါ။");
}

#[test]
fn attached_forms_take_no_particle()
{
    let text = "အကယ်၍ က သည် ၁ အောက် မငယ်သည်လျှင် က ကို ပြသပါ။";
    assert_eq!(warns(text), [Code::ParticleBeforeAttached]);
    clean("အကယ်၍ က သည် ၁ အောက် မငယ်လျှင် က ကို ပြသပါ။");
    clean("အကယ်၍ က သည် ၁ အောက် မငယ်သည် ဖြစ်လျှင် က ကို ပြသပါ။");
    let loop_text = "က သည် ၁ ထက် ကြီးပါနေသရွေ့ က ကို ပြသပါ။";
    assert_eq!(warns(loop_text), [Code::ParticleBeforeAttached]);
    clean("က သည် ၁ ထက် ကြီးနေသရွေ့ က ကို ပြသပါ။");
    clean("က သည် ၁ ထက် ကြီးသည် ဖြစ်နေသရွေ့ က ကို ပြသပါ။");
}

#[test]
fn not_never_takes_thi()
{
    assert_eq!(warns("(က မဟုတ်သည်) ဟု ပြသပါ။"), [Code::NotWithThi]);
    assert_eq!(warns("(က မဟုတ်ပါသည်) ဟု ပြသပါ။"), [Code::NotWithThi]);
    clean("(က မဟုတ်ပါ) ဟု ပြသပါ။");
    clean("(က မဟုတ်) ကို ပြသပါ။");
}

#[test]
fn pa_before_and_wants_thi()
{
    let text = "(က သည် ၁ အောက် မငယ်ပါ ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသပါ။";
    assert_eq!(warns(text), [Code::PaBeforeAnd]);
    clean("(က သည် ၁ အောက် မငယ်သည် ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသပါ။");
    clean("(က သည် ၁ အောက် မငယ် ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသပါ။");
}

#[test]
fn sentences_and_strings_are_quoted()
{
    assert_eq!(warns("က သည် ၁ နှင့် မညီပါ ကို ပြသပါ။"), [Code::PrintSentenceWithKo]);
    assert_eq!(warns("က သည် ၁ နှင့် ညီသည် ကို ပြသပါ။"), [Code::PrintSentenceWithKo]);
    assert_eq!(warns("\"x\" ကို ပြသပါ။"), [Code::PrintSentenceWithKo]);
    assert_eq!(warns("မှန်ပါသည် ကို ပြသပါ။"), [Code::PrintSentenceWithKo]);
    clean("က သည် ၁ နှင့် မညီပါ ဟု ပြသပါ။");
    clean("က သည် ၁ နှင့် ညီသည် ဟု ပြသပါ။");
    clean("\"x\" ဟု ပြသပါ။");
    clean("(က သည် ၁ နှင့် ညီသည် မဟုတ်) ကို ပြသပါ။");
    clean("က သည် ၁ နှင့် ညီ ကို ပြသပါ။");
}

#[test]
fn values_are_marked()
{
    assert_eq!(warns("က ဟု ပြသပါ။"), [Code::PrintValueWithHu]);
    assert_eq!(warns("က + ၁ ဟု ပြသပါ။"), [Code::PrintValueWithHu]);
    assert_eq!(warns("က > ၁ ဟု ပြသပါ။"), [Code::PrintValueWithHu]);
    clean("က ကို ပြသပါ။");
    clean("က + ၁ ကို ပြသပါ။");
    clean("မှန် ကို ပြသပါ။");
}

#[test]
fn strict_reports_errors()
{
    let text = "\"x\" ကို ပြသပါ။";
    let mut sm = SourceMap::new();
    let file = sm.new_source_file(FileName::Custom("t.zaka".into()), text.to_string());
    let lexed = lex(&file, &LexerOptions::default());
    let parsed = parse(&file, &lexed);
    let lax = check(&parsed.program, &NaturalnessOptions::default());
    assert_eq!(lax[0].level, Level::Warning);
    let strict = NaturalnessOptions::builder().strict(true).build();
    let strict = check(&parsed.program, &strict);
    assert_eq!(strict[0].level, Level::Error);
    assert_eq!(strict[0].code, Code::PrintSentenceWithKo);
}

#[test]
fn diagnostics_come_in_source_order()
{
    let text = "\"x\" ကို ပြသပါ။\nက ဟု ပြသပါ။";
    assert_eq!(warns(text), [Code::PrintSentenceWithKo, Code::PrintValueWithHu]);
}

#[test]
fn redundant_parentheses_around_a_whole_operand()
{
    assert_eq!(warns("(က) ကို ပြသပါ။"), [Code::RedundantGroup]);
    assert_eq!(warns("(က + ၁) ကို ပြသပါ။"), [Code::RedundantGroup]);
    assert_eq!(warns("(က သည် ၁ နှင့် မညီပါ) ဟု ပြသပါ။"), [Code::RedundantGroup]);
    assert_eq!(warns("က ကို (ခ + ၁) ဟု ထားပါ။"), [Code::RedundantGroup]);
    assert_eq!(warns("(က + ၁) ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။"), [Code::RedundantGroup]);
    let init = "ကိန်းရှင် က သည် ကိန်းပြည့် (ခ + ၁) ဖြစ်ပါစေ။";
    assert_eq!(warns(init), [Code::RedundantGroup]);
    let bound = "(က + ၁) မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသပါ။";
    assert_eq!(warns(bound), [Code::RedundantGroup]);
    let arg = "လုပ်ဆောင်ချက် f ကို (က + ၁) တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(warns(arg), [Code::RedundantGroup]);
    clean("(က ဖြစ်၍ ခ) ကို ပြသပါ။");
    clean("လုပ်ဆောင်ချက် f ကို က၊ (၁၊ ၂ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်ပါ။");
    clean("(က သည် ၁ နှင့် ညီသည် မဟုတ်) ကို ပြသပါ။");
    clean("က ကို (ခ သို့မဟုတ် ဂ) ဟု ထားပါ။");
}

#[test]
fn redundant_parentheses_around_a_condition()
{
    let text = "အကယ်၍ (က သည် ၁ ထက် ကြီး) လျှင် က ကို ပြသပါ။";
    assert_eq!(warns(text), [Code::RedundantGroup]);
    let logical = "အကယ်၍ (က ဖြစ်၍ ခ) လျှင် က ကို ပြသပါ။";
    assert_eq!(warns(logical), [Code::RedundantGroup]);
    let loop_text = "(က သည် ၁ ထက် ကြီး) နေသရွေ့ က ကို ပြသပါ။";
    assert_eq!(warns(loop_text), [Code::RedundantGroup]);
    clean("အကယ်၍ က သည် ၁ ထက် ကြီးလျှင် က ကို ပြသပါ။");
    clean("အကယ်၍ (က ဖြစ်၍ ခ) မဟုတ်လျှင် က ကို ပြသပါ။");
}

#[test]
fn nested_parentheses_are_redundant()
{
    assert_eq!(warns("((က)) ကို ပြသပါ။"), [Code::RedundantGroup, Code::RedundantGroup]);
    assert_eq!(warns("((က ဖြစ်၍ ခ)) ကို ပြသပါ။"), [Code::RedundantGroup]);
}

#[test]
fn parentheses_inside_chains_are_left_alone()
{
    clean("(က + ၁) * ၂ ကို ပြသပါ။");
    clean("က + (ခ + ၁) ကို ပြသပါ။");
    clean("(က + ၁) နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသပါ။");
    clean("(က သည် ၁ ထက် ကြီးသည်) သည် မှန် နှင့် ညီသည် ဟု ပြသပါ။");
    clean("!(က > ၁ && ခ < ၂) ကို ပြသပါ။");
    clean("အကယ်၍ (က သို့မဟုတ် ခ) ဖြစ်၍ ဂ လျှင် က ကို ပြသပါ။");
}

#[test]
fn address_and_value_verbs_follow_the_particle_rules()
{
    assert!(warns("p ၏ တန်ဖိုး ကို ပြသပါ။").is_empty());
    assert!(warns("x ၏ လိပ်စာ ကို ပြသပါ။").is_empty());
    assert_eq!(warns("(p ၏ တန်ဖိုး) ကို ပြသပါ။"), [Code::RedundantGroup]);
}

#[test]
fn a_clause_used_as_a_value_wants_thi()
{
    let init = "ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး က သည် ၄၀ အောက် မငယ်ပါ ဖြစ်ပါစေ။";
    assert_eq!(warns(init), [Code::PaInValue]);
    clean("ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး က သည် ၄၀ အောက် မငယ်သည် ဖြစ်ပါစေ။");
    clean("ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး က သည် ၄၀ အောက် မငယ် ဖြစ်ပါစေ။");
    let ret = "က သည် ၁ နှင့် မညီပါသည် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။";
    assert_eq!(warns(ret), [Code::PaInValue]);
    clean("က သည် ၁ နှင့် မညီသည် ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။");
    let arg = "လုပ်ဆောင်ချက် f ကို က သည် ၁ နှင့် မညီပါ တို့ဖြင့် လုပ်ဆောင်ပါ။";
    assert_eq!(warns(arg), [Code::PaInValue]);
    let logical = "ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး (က မဟုတ် သို့မဟုတ် က သည် ၁ နှင့် မညီပါ) ဖြစ်ပါစေ။";
    assert_eq!(warns(logical), [Code::PaInValue]);
    clean("ခ ကို က သည် ၄၀ အောက် မငယ်ပါ ဟု ထားပါ။");
    clean("က သည် ၄၀ အောက် မငယ်ပါ ဟု ပြသပါ။");
    clean("ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး မမှန် ဖြစ်ပါစေ။");
}
