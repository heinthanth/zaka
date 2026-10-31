use std::path::Path;
use std::sync::Arc;

use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_semcheck::artifact::check_loaded;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::file_loader::MemFileLoader;
use zakac_span::source_map::SourceMap;

/// Check `/m/main.zaka` and the files it imports, from files held in RAM.
fn check(files: &[(&str, &str)]) -> Vec<Diagnostic>
{
    let mut loader = MemFileLoader::new();
    files.iter().for_each(|(path, text)| loader.insert(*path, text));
    let mut sm = SourceMap::with_loader(Arc::new(loader));
    let file = sm.load_file(Path::new("/m/main.zaka")).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    check_loaded(loaded, &SemcheckOptions::default()).diagnostics
}

/// The codes of [`check`].
fn codes(files: &[(&str, &str)]) -> Vec<Code>
{
    check(files).iter().map(|d| d.code).collect()
}

/// A file with an exported function `ပေါင်း`, a private function `နှစ်ဆ`,
/// an exported constant `အများဆုံး` and a private variable `ဝှက်`.
const LIB: &str = "\
အများသုံး ကိန်းသေ အများဆုံး သည် ကိန်းပြည့် ဖြစ်၍ တန်ဖိုးသည် ၁၀ ဖြစ်ပါစေ။
ကိန်းရှင် ဝှက် သည် ကိန်းပြည့် ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် နှစ်ဆ သည်
ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၂ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
အများသုံး လုပ်ဆောင်ချက် ပေါင်း သည်
ကိန်းပြည့် က၊ ကိန်းပြည့် ခ တို့ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
(လုပ်ဆောင်ချက် နှစ်ဆ ကို က တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်) + ခ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
";

#[test]
fn exported_names_reach_the_importer()
{
    let main = "\"lib.zaka\" မှ ပေါင်း၊ အများဆုံး တို့ကို ထည့်သွင်းပါ။
လုပ်ဆောင်ချက် ပေါင်း ကို အများဆုံး၊ ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။";
    assert_eq!(codes(&[("/m/main.zaka", main), ("/m/lib.zaka", LIB)]), []);
}

#[test]
fn two_files_can_each_have_a_private_function_of_one_name()
{
    let main = "\"lib.zaka\" မှ ပေါင်း ကို ထည့်သွင်းပါ။
လုပ်ဆောင်ချက် နှစ်ဆ သည်
မှန်မှားတန်ဖိုး က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး ရလဒ် ကို ပြန်ပေးမည်။
က ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။
လုပ်ဆောင်ချက် နှစ်ဆ ကို မှန် တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။";
    assert_eq!(codes(&[("/m/main.zaka", main), ("/m/lib.zaka", LIB)]), []);
}

#[test]
fn importing_a_private_name_is_zk0322_with_help()
{
    let main = "\"lib.zaka\" မှ နှစ်ဆ၊ ဝှက်၊ မရှိ တို့ကို ထည့်သွင်းပါ။";
    let found = check(&[("/m/main.zaka", main), ("/m/lib.zaka", LIB)]);
    let codes: Vec<Code> = found.iter().map(|d| d.code).collect();
    assert_eq!(codes, [Code::ImportNotFound; 3]);
    let helped: Vec<bool> = found.iter().map(|d| d.help.is_some()).collect();
    assert_eq!(helped, [true, true, false]);
}

#[test]
fn a_name_that_is_not_imported_is_unknown_with_help()
{
    let main = "\"lib.zaka\" မှ ပေါင်း ကို ထည့်သွင်းပါ။\nဝှက် ကို ပြသပါ။\nလုပ်ဆောင်ချက် နှစ်ဆ ကို ၁ တို့ဖြင့် \
                လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။";
    let found = check(&[("/m/main.zaka", main), ("/m/lib.zaka", LIB)]);
    let codes: Vec<Code> = found.iter().map(|d| d.code).collect();
    assert_eq!(codes, [Code::UndeclaredName, Code::UndeclaredFunction]);
    let helps: Vec<&str> = found.iter().filter_map(|d| d.help.as_deref()).collect();
    assert_eq!(helps.len(), 2);
    assert!(helps.iter().all(|h| h.contains("lib.zaka")), "{helps:?}");
}

#[test]
fn an_imported_name_and_a_declaration_of_one_name_clash()
{
    let main = "\"lib.zaka\" မှ အများဆုံး ကို ထည့်သွင်းပါ။
ကိန်းရှင် အများဆုံး သည် ကိန်းပြည့် ဖြစ်ပါစေ။";
    assert_eq!(
        codes(&[("/m/main.zaka", main), ("/m/lib.zaka", LIB)]),
        [Code::Redeclaration]
    );
}

#[test]
fn a_statement_in_an_imported_file_is_zk0323()
{
    let lib = "အများသုံး ကိန်းရှင် က သည် ကိန်းပြည့် ဖြစ်ပါစေ။\nက ကို ပြသပါ။";
    let main = "\"lib.zaka\" မှ က ကို ထည့်သွင်းပါ။\nက ကို ပြသပါ။";
    assert_eq!(
        codes(&[("/m/main.zaka", main), ("/m/lib.zaka", lib)]),
        [Code::StmtInImportedFile]
    );
}

#[test]
fn an_import_brings_nothing_the_imported_file_imports()
{
    let base = "အများသုံး ကိန်းရှင် စုစုပေါင်း သည် ကိန်းပြည့် ဖြစ်ပါစေ။";
    let mid = "\"base.zaka\" မှ စုစုပေါင်း ကို ထည့်သွင်းပါ။";
    let main = "\"mid.zaka\" မှ စုစုပေါင်း ကို ထည့်သွင်းပါ။";
    let files = [("/m/main.zaka", main), ("/m/mid.zaka", mid), ("/m/base.zaka", base)];
    assert_eq!(codes(&files), [Code::ImportNotFound]);
}

/// A private struct and an exported one.
const STRUCTS: &str = "\
ဒေတာအမျိုးအစား ဝှက် တွင် ကိန်းပြည့် x တို့ ပါဝင်သည်။
အများသုံး ဒေတာအမျိုးအစား နေရာ တွင် ကိန်းပြည့် x တို့ ပါဝင်သည်။
";

/// The codes of `/m/main.zaka` alone, after [`STRUCTS`].
fn alone(text: &str) -> Vec<Code>
{
    let main = format!("{STRUCTS}{text}");
    codes(&[("/m/main.zaka", main.as_str())])
}

#[test]
fn an_export_that_names_a_private_struct_is_zk0324()
{
    let param = "အများသုံး လုပ်ဆောင်ချက် ဖ သည် ဝှက် က ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert_eq!(alone(param), [Code::PrivateInExport]);
    let result = "အများသုံး လုပ်ဆောင်ချက် ဖ သည် အောက်ပါတို့ကို လုပ်ဆောင်ကာ ဝှက် ရလဒ် ကို ပြန်ပေးမည်။
ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးပါပြီ။";
    assert!(alone(result).contains(&Code::PrivateInExport), "{:?}", alone(result));
    let var = "အများသုံး ကိန်းရှင် က သည် ဝှက် လိပ်စာ ဖြစ်ပါစေ။";
    assert_eq!(alone(var), [Code::PrivateInExport]);
    let field = "အများသုံး ဒေတာအမျိုးအစား စာရင်းခေါင်း တွင် ဝှက် လိပ်စာ ပထမ တို့ ပါဝင်သည်။";
    assert_eq!(alone(field), [Code::PrivateInExport]);
}

#[test]
fn an_export_that_names_an_exported_or_imported_struct_passes()
{
    let exported = "အများသုံး ကိန်းရှင် က သည် နေရာ ဖြစ်ပါစေ။";
    assert_eq!(alone(exported), []);
    let private = "ကိန်းရှင် က သည် ဝှက် ဖြစ်ပါစေ။";
    assert_eq!(alone(private), []);
    let lib = "အများသုံး ဒေတာအမျိုးအစား လူ တွင် ကိန်းပြည့် အသက် တို့ ပါဝင်သည်။";
    let main = "\"lib.zaka\" မှ လူ ကို ထည့်သွင်းပါ။\nအများသုံး ကိန်းရှင် က သည် လူ ဖြစ်ပါစေ။";
    assert_eq!(codes(&[("/m/main.zaka", main), ("/m/lib.zaka", lib)]), []);
}
