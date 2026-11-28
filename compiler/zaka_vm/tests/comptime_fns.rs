//! Comptime functions: calls in spots that need a value at compile time run
//! on the VM while the compiler runs, and compile-time conditions can call
//! them.

use std::path::Path;
use std::path::PathBuf;

use zaka_host::Grant;
use zaka_host::Grants;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_with;
use zakac_ast::target::Target;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_codegen_target_vm::artifact::run_loaded;
use zakac_diagnostics::Code;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_semcheck::open_chains::load_settled;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::source_map::SourceMap;

/// Held by each test that sets the grants: the grants are one value for the
/// whole process, so two such tests on threads of one process must not overlap.
static GRANTS_HELD: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Hold the grants for the rest of a test.
///
/// # Returns
///
/// The guard.
fn hold_grants() -> std::sync::MutexGuard<'static, ()>
{
    GRANTS_HELD.lock().unwrap_or_else(|e| e.into_inner())
}

/// The head of a comptime function `နှစ်ဆ` that doubles an integer.
const DOUBLE: &str = "နှစ်ဆ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nn ဟူသော ကိန်းပြည့် \
                      တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nn * ၂ ကို \
                      ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n";

/// A start block that prints a name.
fn printing(name: &str) -> String
{
    format!("ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\n{name} ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n")
}

/// A fresh folder for the files of one test.
fn folder(name: &str) -> PathBuf
{
    let dir = std::env::temp_dir()
        .join(format!("zaka-comptime-{name}-{}", std::process::id()));
    _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Compile the file `main.zaka` of a folder, with the passes that settle
/// conditions, and run it: the codes of its diagnostics, and what it printed.
fn build_and_run(dir: &Path) -> (Vec<Code>, String)
{
    zaka_vm::comptime::install();
    let mut sm = SourceMap::new();
    let file = sm.load_file(&dir.join("main.zaka")).unwrap();
    let options = SemcheckOptions::default();
    let lexer = LexerOptions::default();
    let loaded =
        load_settled(&file, &mut sm, &lexer, &Target::host(), &options).unwrap();
    #[rustfmt::skip]
    let emitted = run_loaded(loaded, &options, &FoldOptions::default(),
        &BorrowCheckOptions::default(), zakac_codegen_target_vm::options::EmitOptions::default());
    let codes = emitted.diagnostics().iter().map(|d| d.code).collect();
    let Some(module) = emitted.module
    else
    {
        return (codes, String::new());
    };
    let mut out = Vec::new();
    run_with(&module, &mut out, &RunOptions::default()).unwrap();
    (codes, String::from_utf8(out).unwrap())
}

/// Write the files of a program into a fresh folder, `main.zaka` among them,
/// then build and run it.
fn program(name: &str, files: &[(&str, &str)]) -> (Vec<Code>, String)
{
    let dir = folder(name);
    for (path, text) in files
    {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    build_and_run(&dir)
}

/// A call in a constant runs while the compiler runs, and its value counts the
/// cells of a list; a call elsewhere runs with the program.
#[test]
fn a_call_in_a_constant_runs_at_compile_time_and_counts_a_list()
{
    let main = format!(
        "{DOUBLE}ကိန်းသေ အရွယ် သည် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် \
         ဖြစ်ပါစေ။\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် ဂ သည် အကွက် အရွယ် ခုရှိသော ကိန်းပြည့် စာရင်း \
         တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ကနဦးတန်ဖိုး ဖြစ်ပါစေ။\nဂ ကို ပြသမည်။\nနှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၂၁ ဖြင့် \
         လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n"
    );
    let (codes, out) = program("count", &[("main.zaka", &main)]);
    assert!(codes.is_empty(), "{codes:?}");
    assert_eq!(out, "[၀၊ ၀၊ ၀၊ ၀]\n၄၂\n");
}

/// Text and lists come back from a run as the values of constants.
#[test]
fn text_and_a_plain_list_come_back_as_constants()
{
    let main = String::from(
        "ခေါင်း ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nn ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး \
         အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို ပြန်ပေးမည်။\n\"အခန်း {n}\" ကို \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nစဉ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော \
         လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် စာရင်း တစ်ခု ကို ပြန်ပေးမည်။\n[၃၊ ၁၊ ၄] ကို \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nကိန်းသေ ခ သည် ခေါင်း ဟူသော လုပ်ဆောင်ချက်ကို ၇ ဖြင့် \
         လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\nကိန်းသေ စ သည် စဉ် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် \
         ဖြစ်ပါစေ။\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nခ ကို ပြသမည်။\nစ ကို ပြသမည်။\nပရိုဂရမ် \
         ပြီးဆုံးပါပြီး။\n",
    );
    let (codes, out) = program("text", &[("main.zaka", &main)]);
    assert!(codes.is_empty(), "{codes:?}");
    assert_eq!(out, "အခန်း ၇\n[၃၊ ၁၊ ၄]\n");
}

/// A variable of an imported file can start from a comptime call.
#[test]
fn an_imported_starting_value_can_come_from_a_comptime_call()
{
    let lib = format!(
        "{DOUBLE}အများသုံးနိုင်သော ကိန်းရှင် အစ သည် နှစ်ဆ ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် \
         ဖြစ်ပါစေ။\n"
    );
    let main = format!("\"lib.zaka\" မှ အစ ကို ထည့်သွင်းမည်။\n{}", printing("အစ"));
    let (codes, out) = program("import", &[("lib.zaka", &lib), ("main.zaka", &main)]);
    assert!(codes.is_empty(), "{codes:?}");
    assert_eq!(out, "၁၀\n");
}

/// An argument that needs the program to run is ZK0608.
#[test]
fn an_argument_not_known_at_compile_time_is_zk0608()
{
    let main = format!(
        "{DOUBLE}ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းရှင် v သည် ၃ ဖြစ်ပါစေ။\nကိန်းသေ က သည် နှစ်ဆ \
         ဟူသော လုပ်ဆောင်ချက်ကို v ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\nက ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n"
    );
    let (codes, _) = program("arg", &[("main.zaka", &main)]);
    assert_eq!(codes, [Code::ComptimeArgNotKnown]);
}

/// A trap, a C call and a write each stop a run or fail inside it; the first
/// two stop the build with ZK0611.
#[test]
fn a_trap_or_a_c_call_stops_the_build_with_zk0611()
{
    let divide = "ခွဲ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nn ဟူသော ကိန်းပြည့် တစ်ခု ကို \
                  လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\n၁၀ / n ကို \
                  ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nကိန်းသေ က သည် ခွဲ ဟူသော လုပ်ဆောင်ချက်ကို \
                  ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\n";
    let (codes, _) =
        program("trap", &[("main.zaka", &format!("{divide}{}", printing("က")))]);
    assert_eq!(codes, [Code::ComptimeRunStopped]);
    let c_call = "abs ဟူသော စိတ်မချရသော လုပ်ဆောင်ချက် သည် \"libc\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍\nn \
                  ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nအပေါင်း \
                  ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် \
                  ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nအောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။\nabs ဟူသော \
                  လုပ်ဆောင်ချက်ကို -၄၂ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ \
                  ပြီးဆုံးပါပြီး။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nကိန်းသေ က သည် အပေါင်း ဟူသော \
                  လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\n";
    let (codes, _) =
        program("c", &[("main.zaka", &format!("{c_call}{}", printing("က")))]);
    assert_eq!(codes, [Code::ComptimeRunStopped]);
}

/// The head of the program of the file tests: a comptime function that reads
/// a file, and one that writes one.
const FILES: &str =
    "\"စံကား/အဝင်အထွက်\" မှ ရေး နှင့် ဖိုင်ကိုဖတ် တို့ကို ထည့်သွင်းမည်။\nဖတ်ယူ ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော \
     လုပ်ဆောင်ချက် သည်\nလမ်း ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို \
     ပြန်ပေးမည်။\nကိန်းရှင် ရ သည် လမ်း ၌ရှိသော ဖိုင်ကိုဖတ် သည့် ရလဒ် ဖြစ်ပါစေ။\nအကယ်၍ ရ ထဲတွင် ရလဒ်ကောင်း ရှိလျှင် \
     ရ ထဲမှ ဒေတာ ကို ပြန်ပေးမည်။\n\"မရ\" ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nရေးကြည့် \
     ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ စာသား တစ်ခု ကို \
     ပြန်ပေးမည်။\nကိန်းရှင် ရ သည် \"x\" ကို \"out.txt\" သို့ ရေး သည့် ရလဒ် ဖြစ်ပါစေ။\nအကယ်၍ ရ ထဲတွင် \
     ရလဒ်ကောင်း ရှိလျှင် \"ရေးပြီး\" ကို ပြန်ပေးမည်။\n\"ရေးမရ\" ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ \
     ပြီးဆုံးပါပြီး။\n";

/// A run with the read right reads the files under the folder of the
/// program.
#[test]
fn a_run_with_the_read_right_reads_a_file()
{
    let _held = hold_grants();
    zakac_semcheck::comptime_calls::set_grants(
        Grants::builder().read(Grant::All).build(),
    );
    let main = format!(
        "{FILES}ကိန်းသေ ဖ သည် ဖတ်ယူ ဟူသော လုပ်ဆောင်ချက်ကို \"data.txt\" ဖြင့် လုပ်ဆောင်သော ရလဒ် \
         ဖြစ်ပါစေ။\n{}",
        printing("ဖ")
    );
    let (codes, out) =
        program("read", &[("data.txt", "from disk"), ("main.zaka", &main)]);
    assert!(codes.is_empty(), "{codes:?}");
    assert_eq!(out, "from disk\n");
}

/// With no right, a read stops the run: ZK0611.
#[test]
fn a_run_with_no_right_cannot_read()
{
    let _held = hold_grants();
    zakac_semcheck::comptime_calls::set_grants(Grants::default());
    let main = format!(
        "{FILES}ကိန်းသေ ဖ သည် ဖတ်ယူ ဟူသော လုပ်ဆောင်ချက်ကို \"data.txt\" ဖြင့် လုပ်ဆောင်သော ရလဒ် \
         ဖြစ်ပါစေ။\n{}",
        printing("ဖ")
    );
    let (codes, _) =
        program("noread", &[("data.txt", "from disk"), ("main.zaka", &main)]);
    assert_eq!(codes, [Code::ComptimeRunStopped]);
}

/// A write with the read right alone stops the run: ZK0611.
#[test]
fn a_write_needs_the_write_right()
{
    let _held = hold_grants();
    zakac_semcheck::comptime_calls::set_grants(
        Grants::builder().read(Grant::All).build(),
    );
    let main = format!(
        "{FILES}ကိန်းသေ ရ သည် ရေးကြည့် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\n{}",
        printing("ရ")
    );
    let (codes, _) = program("write", &[("main.zaka", &main)]);
    assert_eq!(codes, [Code::ComptimeRunStopped]);
}

/// What a comptime call prints is a note at the call: ZK0612.
#[test]
fn a_print_at_compile_time_is_a_note()
{
    let main = format!(
        "ပြော ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် \
         တစ်ခု ကို ပြန်ပေးမည်။\n\"hi\" ဟု ပြသမည်။\n၁ ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ \
         ပြီးဆုံးပါပြီး။\nကိန်းသေ က သည် ပြော ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\n{}",
        printing("က")
    );
    let (codes, out) = program("print", &[("main.zaka", &main)]);
    assert_eq!(codes, [Code::ComptimePrinted]);
    assert_eq!(out, "၁\n");
}

/// A condition calls a comptime function; a chain whose callee an arm of
/// another declares waits a pass; a nested chain settles in a later pass.
#[test]
fn conditions_that_call_comptime_functions_settle_in_passes()
{
    let main =
        "ကျယ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nn ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး \
         အောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။\nn > ၁၀ ကို \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nအကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် ကျယ် ဟူသော လုပ်ဆောင်ချက်ကို \
         ၂၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nထပ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် \
         လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။\nမှန် ကို \
         ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nအကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် ကျယ် ဟူသော လုပ်ဆောင်ချက်ကို \
         ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းသေ အတွင်း သည် \"ကျယ်\" \
         ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nသို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းသေ အတွင်း \
         သည် \"ကျဉ်း\" ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nလုပ်ဆောင်စရာများ \
         ပြီးဆုံးပါပြီး။\nသို့မဟုတ်ပါက အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းသေ အတွင်း သည် \"-\" \
         ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nအကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို \
         လုပ်ဆောင်သော ရလဒ် ဖြစ်၍ အလုပ်လုပ်မည့်ပလက်ဖောင်းဘစ် == ၆၄ ဖြစ်လျှင် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nကိန်းသေ \
         ဒုတိယ သည် \"ရ\" ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nသို့မဟုတ်ပါက အောက်ပါတို့ကို \
         လုပ်ဆောင်မည်။\nကိန်းသေ ဒုတိယ သည် \"မရ\" ဖြစ်ပါစေ။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nပရိုဂရမ်သည် \
         အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nအတွင်း ကို ပြသမည်။\nဒုတိယ ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let (codes, out) = program("passes", &[("main.zaka", main)]);
    assert!(codes.is_empty(), "{codes:?}");
    assert_eq!(out, "ကျဉ်း\nရ\n");
}

/// A condition that calls a function its own chain declares waits on itself:
/// ZK0609.
#[test]
fn a_callee_in_its_own_chain_is_zk0609()
{
    let main = "အကယ်၍ ကွန်ပိုင်းလုပ်ချိန်တွင် ကိုယ်တိုင် ဟူသော လုပ်ဆောင်ချက်ကို လုပ်ဆောင်သော ရလဒ် ဖြစ်လျှင် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nကိုယ်တိုင် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို \
                လုပ်ဆောင်ကာ မှန်မှားတန်ဖိုး တစ်ခု ကို ပြန်ပေးမည်။\nမှန် ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ \
                ပြီးဆုံးပါပြီး။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\nပရိုဂရမ်သည် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let (codes, _) = program("own", &[("main.zaka", main)]);
    assert_eq!(codes, [Code::ComptimeWaitsOnItself]);
}

/// A call through an operator on a struct reaches its function with no name
/// written, and an instance of a generic gives its own result kind: each
/// stays in its small program.
#[test]
fn an_operator_and_a_generic_run_at_compile_time()
{
    let main = "အလံ ဟူသော ဒေတာအစုအဝေးတွင် ဘစ်များ ဟူသော ကိန်းပြည့် တစ်ခု ပါဝင်သည်။\n\nဘစ်ဆုံ ဟူသော \
                လုပ်ဆောင်ချက် သည်\nက ဟူသော အလံ တစ်ခု ကို လည်းကောင်း ခ ဟူသော အလံ တစ်ခု ကို \
                လည်းကောင်း\nလက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ အလံ တစ်ခု ကို ပြန်ပေးမည်။\nအလံ(ဘစ်များ: (က \
                ၏ ဘစ်များ) & (ခ ၏ ဘစ်များ)) ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n\nဆုံ \
                ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည်\nအောက်ပါတို့ကို လုပ်ဆောင်ကာ အလံ တစ်ခု \
                ကို ပြန်ပေးမည်။\nအလံ(ဘစ်များ: ၁၂) & အလံ(ဘစ်များ: ၁၀) ကို \
                ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n\nထပ် ဟူသော ကွန်ပိုင်းလုပ်ချိန်တွင် \
                လုပ်ဆောင်နိုင်သော လုပ်ဆောင်ချက် သည် ယေဘုယျဒေတာအမျိုးအစား T ကိုအစားထိုးသုံးနိုင်၍ ၎င်းသည်\nက \
                ဟူသော T တစ်ခု ကို လက်ခံပြီး အောက်ပါတို့ကို လုပ်ဆောင်ကာ T တစ်ခု ကို ပြန်ပေးမည်။\nက ကို \
                ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။\n\nကိန်းသေ ဆ သည် ဆုံ ဟူသော လုပ်ဆောင်ချက်ကို \
                လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\nကိန်းသေ ထ သည် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို \"ပြန်\" ဖြင့် \
                လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\nကိန်းသေ ထ၂ သည် ထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၄၂ ဖြင့် \
                လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။\n\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nဆ ကို ပြသမည်။\nထ ကို \
                ပြသမည်။\nထ၂ ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    let (codes, out) = program("gaps", &[("main.zaka", main)]);
    assert!(codes.is_empty(), "{codes:?}");
    assert_eq!(out, "အလံ(ဘစ်များ: ၈)\nပြန်\n၄၂\n");
}
