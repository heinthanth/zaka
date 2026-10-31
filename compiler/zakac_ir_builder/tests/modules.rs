use std::path::Path;
use std::sync::Arc;

use zakac_diagnostics::Code;
use zakac_ir_builder::artifact::Lowered;
use zakac_ir_builder::artifact::run_loaded;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::file_loader::MemFileLoader;
use zakac_span::source_map::SourceMap;

/// Lower and fold `/m/main.zaka` and the file `/m/lib.zaka` it imports.
fn lower_pair(main: &str, lib: &str) -> Lowered
{
    let mut loader = MemFileLoader::new();
    loader.insert("/m/main.zaka", main);
    loader.insert("/m/lib.zaka", lib);
    let mut sm = SourceMap::with_loader(Arc::new(loader));
    let file = sm.load_file(Path::new("/m/main.zaka")).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    run_loaded(loaded, &SemcheckOptions::default(), &FoldOptions::default())
}

/// The codes of [`lower_pair`].
fn codes(main: &str, lib: &str) -> Vec<Code>
{
    lower_pair(main, lib).diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn imported_values_known_at_compile_time_pass()
{
    let lib = "\
အများသုံး ကိန်းသေ က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ * ၂ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ဖ သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
အများသုံး ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က + ၁ ဖြစ်ပါစေ။
အများသုံး ကိန်းရှင် ဂ သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် စာရင်း ဖြစ်၍ တန်ဖိုးသည် ၁၊ က တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် ကိန်းပြည့် ဖြစ်ပါစေ။
";
    let main = "\"lib.zaka\" မှ က၊ ခ၊ ဂ တို့ကို ထည့်သွင်းပါ။\nက + ခ ကို ပြသပါ။";
    let lowered = lower_pair(main, lib);
    let found: Vec<Code> = lowered.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(found, []);
    let module = lowered.folded.expect("lowered");
    assert_eq!(module.imported_inits, 4);
    assert_eq!(module.main.len(), 5);
}

#[test]
fn an_imported_value_from_a_call_is_zk0604()
{
    let lib = "\
လုပ်ဆောင်ချက် တစ် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
အများသုံး ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် တစ် ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
";
    let main = "\"lib.zaka\" မှ က ကို ထည့်သွင်းပါ။\nက ကို ပြသပါ။";
    assert_eq!(codes(main, lib), [Code::GlobalInitNotConstant; 2]);
}

#[test]
fn the_file_that_is_run_keeps_values_from_calls()
{
    let lib = "\
အများသုံး လုပ်ဆောင်ချက် တစ် သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
၁ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";
    let main = "\"lib.zaka\" မှ တစ် ကို ထည့်သွင်းပါ။
ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် လုပ်ဆောင်ချက် တစ် ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
က ကို ပြသပါ။";
    let lowered = lower_pair(main, lib);
    assert!(lowered.diagnostics.is_empty(), "{:?}", lowered.diagnostics);
    let module = lowered.folded.expect("lowered");
    assert_eq!(module.imported_inits, 0);
    let names: Vec<&str> = module.funcs.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["တစ်@lib"]);
}

#[test]
fn the_check_of_imported_values_does_not_hang_on_the_fold_options()
{
    let lib = "အများသုံး ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ * ၃ ဖြစ်ပါစေ။";
    let main = "\"lib.zaka\" မှ က ကို ထည့်သွင်းပါ။\nက ကို ပြသပါ။";
    let mut loader = MemFileLoader::new();
    loader.insert("/m/main.zaka", main);
    loader.insert("/m/lib.zaka", lib);
    let mut sm = SourceMap::with_loader(Arc::new(loader));
    let file = sm.load_file(Path::new("/m/main.zaka")).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    let lowered = run_loaded(loaded, &SemcheckOptions::default(), &FoldOptions::none());
    assert!(lowered.diagnostics.is_empty(), "{:?}", lowered.diagnostics);
}
