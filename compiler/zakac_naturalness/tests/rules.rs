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
    let text = "(က သည် ၁ အောက် မငယ်သည် သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသမည်။";
    assert_eq!(warns(text), [Code::NegativeBeforeOr]);
    clean("(က သည် ၁ အောက် မငယ်ပါ သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသမည်။");
    clean("(က သည် ၁ အောက် ငယ်သည် သို့မဟုတ် က သည် ၂ ထက် ကြီးသည်) ဟု ပြသမည်။");
    assert_eq!(warns("(မမှန်သည် သို့မဟုတ် မှန်သည်) ဟု ပြသမည်။"), [Code::NegativeBeforeOr]);
    clean("(က မဟုတ် သို့မဟုတ် ခ) ကို ပြသမည်။");
}

#[test]
fn attached_forms_take_no_particle()
{
    let text = "အကယ်၍ က သည် ၁ အောက် မငယ်သည်လျှင် က ကို ပြသမည်။";
    assert_eq!(warns(text), [Code::ParticleBeforeAttached]);
    clean("အကယ်၍ က သည် ၁ အောက် မငယ်လျှင် က ကို ပြသမည်။");
    clean("အကယ်၍ က သည် ၁ အောက် မငယ်သည် ဖြစ်လျှင် က ကို ပြသမည်။");
    let loop_text = "က သည် ၁ ထက် ကြီးပါနေသရွေ့ က ကို ပြသမည်။";
    assert_eq!(warns(loop_text), [Code::ParticleBeforeAttached]);
    clean("က သည် ၁ ထက် ကြီးနေသရွေ့ က ကို ပြသမည်။");
    clean("က သည် ၁ ထက် ကြီးသည် ဖြစ်နေသရွေ့ က ကို ပြသမည်။");
}

#[test]
fn not_never_takes_thi()
{
    assert_eq!(warns("(က မဟုတ်သည်) ဟု ပြသမည်။"), [Code::NotWithThi]);
    assert_eq!(warns("(က မဟုတ်ပါသည်) ဟု ပြသမည်။"), [Code::NotWithThi]);
    clean("(က မဟုတ်ပါ) ဟု ပြသမည်။");
    clean("(က မဟုတ်) ကို ပြသမည်။");
}

#[test]
fn pa_before_and_wants_thi()
{
    let text = "(က သည် ၁ အောက် မငယ်ပါ ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသမည်။";
    assert_eq!(warns(text), [Code::PaBeforeAnd]);
    clean("(က သည် ၁ အောက် မငယ်သည် ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသမည်။");
    clean("(က သည် ၁ အောက် မငယ် ဖြစ်၍ က သည် ၂ အောက် ငယ်သည်) ဟု ပြသမည်။");
}

#[test]
fn sentences_and_strings_are_quoted()
{
    assert_eq!(warns("က သည် ၁ နှင့် မညီပါ ကို ပြသမည်။"), [Code::PrintSentenceWithKo]);
    assert_eq!(warns("က သည် ၁ နှင့် ညီသည် ကို ပြသမည်။"), [Code::PrintSentenceWithKo]);
    assert_eq!(warns("\"x\" ကို ပြသမည်။"), [Code::PrintSentenceWithKo]);
    assert_eq!(warns("မှန်ပါသည် ကို ပြသမည်။"), [Code::PrintSentenceWithKo]);
    clean("က သည် ၁ နှင့် မညီပါ ဟု ပြသမည်။");
    clean("က သည် ၁ နှင့် ညီသည် ဟု ပြသမည်။");
    clean("\"x\" ဟု ပြသမည်။");
    clean("(က သည် ၁ နှင့် ညီသည် မဟုတ်) ကို ပြသမည်။");
    clean("က သည် ၁ နှင့် ညီ ကို ပြသမည်။");
}

#[test]
fn values_are_marked()
{
    assert_eq!(warns("က ဟု ပြသမည်။"), [Code::PrintValueWithHu]);
    assert_eq!(warns("က + ၁ ဟု ပြသမည်။"), [Code::PrintValueWithHu]);
    assert_eq!(warns("က > ၁ ဟု ပြသမည်။"), [Code::PrintValueWithHu]);
    clean("က ကို ပြသမည်။");
    clean("က + ၁ ကို ပြသမည်။");
    clean("မှန် ကို ပြသမည်။");
}

#[test]
fn strict_reports_errors()
{
    let text = "\"x\" ကို ပြသမည်။";
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
    let text = "\"x\" ကို ပြသမည်။\nက ဟု ပြသမည်။";
    assert_eq!(warns(text), [Code::PrintSentenceWithKo, Code::PrintValueWithHu]);
}

#[test]
fn redundant_parentheses_around_a_whole_operand()
{
    assert_eq!(warns("(က) ကို ပြသမည်။"), [Code::RedundantGroup]);
    assert_eq!(warns("(က + ၁) ကို ပြသမည်။"), [Code::RedundantGroup]);
    assert_eq!(warns("(က သည် ၁ နှင့် မညီပါ) ဟု ပြသမည်။"), [Code::RedundantGroup]);
    assert_eq!(warns("က ကို (ခ + ၁) ဟု ထားမည်။"), [Code::RedundantGroup]);
    assert_eq!(warns("(က + ၁) ကို ပြန်ပေးမည်။"), [Code::RedundantGroup]);
    let init = "ကိန်းရှင် က သည် ကိန်းပြည့် တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (ခ + ၁) ဖြစ်ပါစေ။";
    assert_eq!(warns(init), [Code::RedundantGroup]);
    let bound = "(က + ၁) မှ ၁၀ အထိ ကိန်း အနေဖြင့် ရေတွက်၍ ကိန်း ကို ပြသမည်။";
    assert_eq!(warns(bound), [Code::RedundantGroup]);
    let arg = "လုပ်ဆောင်ချက် f ကို (က + ၁) တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(warns(arg), [Code::RedundantGroup]);
    clean("(က ဖြစ်၍ ခ) ကို ပြသမည်။");
    clean("လုပ်ဆောင်ချက် f ကို က၊ (၁၊ ၂ တို့ပါသော စာရင်း) တို့ဖြင့် လုပ်ဆောင်မည်။");
    clean("(က သည် ၁ နှင့် ညီသည် မဟုတ်) ကို ပြသမည်။");
    clean("က ကို (ခ သို့မဟုတ် ဂ) ဟု ထားမည်။");
}

#[test]
fn redundant_parentheses_around_a_condition()
{
    let text = "အကယ်၍ (က သည် ၁ ထက် ကြီး) လျှင် က ကို ပြသမည်။";
    assert_eq!(warns(text), [Code::RedundantGroup]);
    let logical = "အကယ်၍ (က ဖြစ်၍ ခ) လျှင် က ကို ပြသမည်။";
    assert_eq!(warns(logical), [Code::RedundantGroup]);
    let loop_text = "(က သည် ၁ ထက် ကြီး) နေသရွေ့ က ကို ပြသမည်။";
    assert_eq!(warns(loop_text), [Code::RedundantGroup]);
    clean("အကယ်၍ က သည် ၁ ထက် ကြီးလျှင် က ကို ပြသမည်။");
    clean("အကယ်၍ (က ဖြစ်၍ ခ) မဟုတ်လျှင် က ကို ပြသမည်။");
}

#[test]
fn nested_parentheses_are_redundant()
{
    assert_eq!(warns("((က)) ကို ပြသမည်။"), [Code::RedundantGroup, Code::RedundantGroup]);
    assert_eq!(warns("((က ဖြစ်၍ ခ)) ကို ပြသမည်။"), [Code::RedundantGroup]);
}

#[test]
fn parentheses_inside_chains_are_left_alone()
{
    clean("(က + ၁) * ၂ ကို ပြသမည်။");
    clean("က + (ခ + ၁) ကို ပြသမည်။");
    clean("(က + ၁) နှင့် ခ ၏ ပေါင်းလဒ် ကို ပြသမည်။");
    clean("(က သည် ၁ ထက် ကြီးသည်) သည် မှန် နှင့် ညီသည် ဟု ပြသမည်။");
    clean("!(က > ၁ && ခ < ၂) ကို ပြသမည်။");
    clean("အကယ်၍ (က သို့မဟုတ် ခ) ဖြစ်၍ ဂ လျှင် က ကို ပြသမည်။");
}

#[test]
fn address_and_value_verbs_follow_the_particle_rules()
{
    assert!(warns("p ၏ တန်ဖိုး ကို ပြသမည်။").is_empty());
    assert!(warns("x ၏ လိပ်စာ ကို ပြသမည်။").is_empty());
    assert_eq!(warns("(p ၏ တန်ဖိုး) ကို ပြသမည်။"), [Code::RedundantGroup]);
}

#[test]
fn a_clause_used_as_a_value_wants_thi()
{
    let init = "ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က သည် ၄၀ အောက် မငယ်ပါ ဖြစ်ပါစေ။";
    assert_eq!(warns(init), [Code::PaInValue]);
    clean("ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က သည် ၄၀ အောက် မငယ်သည် ဖြစ်ပါစေ။");
    clean("ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က သည် ၄၀ အောက် မငယ် ဖြစ်ပါစေ။");
    let ret = "က သည် ၁ နှင့် မညီပါသည် ကို ပြန်ပေးမည်။";
    assert_eq!(warns(ret), [Code::PaInValue]);
    clean("က သည် ၁ နှင့် မညီသည် ကို ပြန်ပေးမည်။");
    let arg = "လုပ်ဆောင်ချက် f ကို က သည် ၁ နှင့် မညီပါ တို့ဖြင့် လုပ်ဆောင်မည်။";
    assert_eq!(warns(arg), [Code::PaInValue]);
    let logical =
        "ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (က မဟုတ် သို့မဟုတ် က သည် ၁ နှင့် မညီပါ) ဖြစ်ပါစေ။";
    assert_eq!(warns(logical), [Code::PaInValue]);
    clean("ခ ကို က သည် ၄၀ အောက် မငယ်ပါ ဟု ထားမည်။");
    clean("က သည် ၄၀ အောက် မငယ်ပါ ဟု ပြသမည်။");
    clean("ကိန်းရှင် ခ သည် မှန်မှားတန်ဖိုး တစ်ခု ဖြစ်၍ တန်ဖိုးသည် မမှန် ဖြစ်ပါစေ။");
}

/// The particle before `သုံး၍` is `ကို` after one argument and `တို့ကို`
/// after several, in a statement and in a value alike.
#[test]
fn arguments_first_take_the_particle_of_their_count()
{
    assert_eq!(warns("က၊ ခ ကို သုံး၍ ပြ မည်။"), [Code::UsingParticleCount]);
    assert_eq!(warns("က တို့ကို သုံး၍ ပြ မည်။"), [Code::UsingParticleCount]);
    let value = "က၊ ခ ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။";
    assert_eq!(warns(value), [Code::UsingParticleCount]);
    clean("က၊ ခ တို့ကို သုံး၍ ပြ မည်။");
    clean("က ကို သုံး၍ လုပ်ဆောင်ချက် ပြ ကို လုပ်ဆောင်မည်။");
    clean("က၊ ခ တို့ကို သုံး၍ ပေါင်း သည့် ရလဒ် ကို ပြသမည်။");
    clean("ပြ မည်။");
}

/// The result of a function kind takes the classifier of its kind, as a
/// function header does, wherever the function kind stands.
#[test]
fn function_kinds_check_the_classifier_of_their_result()
{
    let head = "(ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ယောက် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား)";
    let param = format!(
        "လုပ်ဆောင်ချက် သုံး သည်\nဖ ဟူသော {head} တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
         လုပ်ဆောင်မည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n"
    );
    assert_eq!(warns(&param), [Code::ClassifierMismatch]);
    let fine = param.replace("တစ်ယောက် ကို ပြန်ပေးသော", "တစ်ခု ကို ပြန်ပေးသော");
    clean(&fine);
}
