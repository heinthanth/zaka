//! A program that calls a C lib of its own, `inputs/ffi_lib.c`, built for
//! the test: a struct through `...`, text as a result, and a function C calls
//! back. It prints the same in the VM, which opens the lib through libffi,
//! and as an executable linked to it.

mod common;

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use common::c_files;
use common::cc;
use common::root;
use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zakac_borrow_checker::options::BorrowCheckOptions;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::driver::write_cache;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::toolchain;
use zakac_codegen_target_c::toolchain::BuildPlan;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_codegen_target_vm::artifact::run_loaded;
use zakac_codegen_target_vm::options::EmitOptions;
use zakac_ir_builder::ir::FoldOptions;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::load;
use zakac_semcheck::semcheck::SemcheckOptions;
use zakac_span::source_map::SourceMap;

/// The program; `{LIB}` stands for the path of the lib, beside it.
const PROGRAM: &str = "\
ဒေတာအမျိုးအစား အမှတ် တွင် ၃၂ ဘစ် ကိန်းပြည့် a၊ ၃၂ ဘစ် ကိန်းပြည့် b တို့ ပါဝင်သည်။

လုပ်ဆောင်ချက် point_sum သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
၃၂ ဘစ် ကိန်းပြည့် count နှင့် အခြားတန်ဖိုးများ ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် name_or_null သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
၃၂ ဘစ် ကိန်းပြည့် good ကို လက်ခံပြီး စာသား ရလဒ် သို့မဟုတ် မအောင်မြင်သည့်သတင်း ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် call_twice သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
(ကိန်းပြည့် ကို လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) f၊ ကိန်းပြည့် v တို့ကို
လက်ခံပြီး ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။

လုပ်ဆောင်ချက် သုံးဆ သည် ကိန်းပြည့် က ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် ရလဒ် ကို ပြန်ပေးမည်။
က * ၃ ကို ရလဒ်အဖြစ် ပြန်ပေးပါ။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
လုပ်ဆောင်ချက် point_sum ကို ၂၊ အမှတ်(a: ၁၊ b: ၂)၊ အမှတ်(a: ၃၊ b: ၄) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် name_or_null ကို ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
လုပ်ဆောင်ချက် name_or_null ကို ၀ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ကိန်းရှင် အမည် သည် လုပ်ဆောင်ချက် name_or_null ကို ၁ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် စာ သည် အမည် ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ် ဖြစ်ပါစေ။
\"{စာ}!\" ဟု ပြသပါ။
လုပ်ဆောင်ချက် call_twice ကို လုပ်ဆောင်ချက် သုံးဆ ၏ လိပ်စာ၊ ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသပါ။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// What the program prints.
const WANT: &str = "၄၆\nရလဒ်ကောင်း(\"zaka\")\nမအောင်မြင်သည့်သတင်း\nzaka!\n၄၅\n";

/// Build the test lib into a folder.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `dir` - The folder.
///
/// # Returns
///
/// The path of the lib.
fn build_lib(cc: &Tool, dir: &Path) -> PathBuf
{
    let name = format!(
        "{}zktest{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let lib = dir.join(name);
    let source = root().join("tests/inputs/ffi_lib.c");
    let mut cmd = Command::new(&cc.program);
    cmd.args(&cc.lead).args(["-shared", "-fPIC", "-o"]).arg(&lib).arg(source);
    // macOS finds a linked lib through the name the lib gives itself.
    if cfg!(target_os = "macos")
    {
        cmd.arg(format!("-Wl,-install_name,{}", lib.display()));
    }
    let built = cmd.output().unwrap();
    let errors = String::from_utf8_lossy(&built.stderr);
    assert!(built.status.success(), "the test lib did not build:\n{errors}");
    lib
}

/// Run the program in the VM.
///
/// # Arguments
///
/// * `path` - The program.
///
/// # Returns
///
/// What it printed.
fn run_in_vm(path: &Path) -> String
{
    let mut sm = SourceMap::new();
    let file = sm.load_file(path).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    let mem = BorrowCheckOptions::default();
    let options = SemcheckOptions::default();
    let emitted = run_loaded(
        loaded,
        &options,
        &FoldOptions::default(),
        &mem,
        EmitOptions::default(),
    );
    let msgs: Vec<&str> =
        emitted.diagnostics().iter().map(|d| d.message.as_str()).collect();
    assert!(emitted.diagnostics().is_empty(), "{msgs:?}");
    let module = emitted.module.expect("emitted");
    let mut out = Vec::new();
    let host = HostData::program("main.zaka");
    let ran = run_and_keep(&module, &mut out, &RunOptions::default(), &host).1;
    let out = String::from_utf8(out).unwrap();
    assert_eq!(ran, Ok(()), "the VM trapped after:\n{out}");
    out
}

/// Build the program as an executable linked to the lib, and run it.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `path` - The program.
/// * `lib` - The lib.
///
/// # Returns
///
/// What it printed.
fn run_native(cc: &Tool, path: &Path, lib: &Path) -> String
{
    let files = c_files(path, OverflowMode::Stop, Checks::Default).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let sources = write_cache(dir.path(), &files).unwrap();
    let exe = dir.path().join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let folder = lib.parent().unwrap().display();
    let mut libs = vec![lib.display().to_string()];
    // Windows finds a lib in the folder of the executable or on the PATH.
    if !cfg!(windows)
    {
        libs.push(format!("-Wl,-rpath,{folder}"));
    }
    #[rustfmt::skip]
    let plan = BuildPlan::builder().cc(cc).sources(&sources).out(&exe)
        .checks(Checks::Default).sanitizers(false).libs(&libs).build();
    let built = toolchain::build(&plan).unwrap();
    let errors: Vec<_> =
        built.outputs.iter().map(|o| String::from_utf8_lossy(&o.stderr)).collect();
    assert!(built.failed.is_none(), "the C build failed:\n{}", errors.concat());
    let ran = Command::new(&exe).stdin(Stdio::null()).output().unwrap();
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert!(ran.status.success(), "the executable stopped:\n{stderr}");
    String::from_utf8(ran.stdout).unwrap()
}

/// The program prints the same in the VM and as an executable.
#[test]
fn a_program_calls_a_lib_of_its_own_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let path = dir.path().join("main.zaka");
    let text = PROGRAM.replace("{LIB}", &lib.display().to_string().replace('\\', "/"));
    std::fs::write(&path, text).unwrap();
    assert_eq!(run_in_vm(&path), WANT, "in the VM");
    assert_eq!(run_native(&cc, &path, &lib), WANT, "as an executable");
}
