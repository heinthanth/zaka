//! A program that calls a C lib of its own, `inputs/ffi_lib.c`, built for
//! the test: a struct through `...`, text as a result, and functions C calls
//! back with numbers, text, structs and handles. It prints the same in the
//! VM, which opens the lib through libffi, and as an executable linked to it.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

use zaka_vm::machine::HostData;
use zaka_vm::machine::RunOptions;
use zaka_vm::machine::run_and_keep;
use zaka_vm::trap::Trap;
use zaka_vm::trap::TrapKind;
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

use crate::common::c_files;
use crate::common::cc;
use crate::common::root;
use crate::common::stop_line;

/// The program; `{LIB}` stands for the path of the lib, beside it.
const PROGRAM: &str = "\
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် a ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် b ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
ရေတွက်စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင်
count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု၊ label ဟူသော စာသား တစ်ခု၊ marks \
                       ဟူသော အကွက် ၂ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း တစ်ခု နှင့် self_ref ဟူသော \
                       ရေတွက်စက် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် counter_open သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_next သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_close သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။
လုပ်ဆောင်ချက် counter_steps သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို \
                       ပြန်ပေးမည်။

လုပ်ဆောင်ချက် point_sum သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း အခြားတန်ဖိုးများနှင့်အတူ လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် name_or_null သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
good ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် point_swap သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
p ဟူသော အမှတ် တစ်ခု ကို ဒေတာ ပြောင်းလဲရန် လက်ခံမည်။
လုပ်ဆောင်ချက် call_twice သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လည်းကောင်း \
                       v ဟူသော ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း
လက်ခံပြီး ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။

လုပ်ဆောင်ချက် သုံးဆ သည် က ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
က * ၃ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
point_sum ဟူသော လုပ်ဆောင်ချက်ကို ၂၊ အမှတ်(a: ၁၊ b: ၂) နှင့် အမှတ်(a: ၃၊ b: ၄) တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို \
                       ပြသမည်။
name_or_null ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
name_or_null ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ကိန်းရှင် အမည် သည် name_or_null ဟူသော လုပ်ဆောင်ချက်ကို ၁ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် စာ သည် အမည် ထဲမှ ဒေတာ ဖြစ်ပါစေ။
\"{စာ}!\" ဟု ပြသမည်။
call_twice ဟူသော လုပ်ဆောင်ချက်ကို သုံးဆ ဟူသော လုပ်ဆောင်ချက် နှင့် ၅ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
ကိန်းရှင် ပ သည် အမှတ်(a: ၁၊ b: ၂) ဖြစ်ပါစေ။
point_swap ဟူသော လုပ်ဆောင်ချက်ကို ပ ဖြင့် လုပ်ဆောင်မည်။
ပ ကို ပြသမည်။
ကိန်းရှင် မရ သည် counter_open ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
မရ ထဲတွင် ဒေတာ ရှိ ကို ပြသမည်။
ကိန်းရှင် စက် သည် counter_open ဟူသော လုပ်ဆောင်ချက်ကို ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ထဲမှ ဒေတာ ဖြစ်ပါစေ။
ကိန်းရှင် တူ သည် စက် ဖြစ်ပါစေ။
counter_next ဟူသော လုပ်ဆောင်ချက်ကို တူ ဖြင့် လုပ်ဆောင်မည်။
\"{စက် ၏ count} {စက် ၏ step}\" ဟု ပြသမည်။
စက် ၏ label ကို ပြသမည်။
စက် ၏ marks ကို ပြသမည်။
ကိန်းရှင် ကိုယ် သည် (စက် ၏ self_ref) ထဲမှ ဒေတာ ဖြစ်ပါစေ။
ကိုယ် ၏ count ကို ပြသမည်။
counter_steps ဟူသော လုပ်ဆောင်ချက်ကို ၂၊ စက် နှင့် တူ တို့ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။
အောက်ပါတို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်။
စက် ၏ marks ကို [၃၊ ၄] ဟု ထားမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
တူ ၏ marks ကို ပြသမည်။
counter_close ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// What the program prints.
const WANT: &str = "၄၆\nဒေတာ(\"zaka\")\nအလွတ်\nzaka!\n၄၅\nအမှတ်(a: ၂၊ b: ၁)\nမမှန်\n၅ \
                    ၅\nဒေတာ(\"big\")\n[၅၊ ၁၀]\n၅\n၁၀\n[၃၊ ၄]\n";

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
    let (out, ran) = vm_run(path);
    assert_eq!(ran, Ok(()), "the VM trapped after:\n{out}");
    out
}

/// Run the program in the VM, which can stop it.
///
/// # Arguments
///
/// * `path` - The program.
///
/// # Returns
///
/// What it printed, and how the run ended.
fn vm_run(path: &Path) -> (String, Result<(), Trap>)
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
    (String::from_utf8(out).unwrap(), ran)
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
    let ran = native_run(cc, path, lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert!(ran.status.success(), "the executable stopped:\n{stderr}");
    String::from_utf8(ran.stdout).unwrap()
}

/// Build the program as an executable linked to the lib, and run it; the
/// run can stop.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `path` - The program.
/// * `lib` - The lib.
///
/// # Returns
///
/// How the run ended, and what it wrote.
fn native_run(cc: &Tool, path: &Path, lib: &Path) -> Output
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
    Command::new(&exe).stdin(Stdio::null()).output().unwrap()
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

/// A program that reads a field through a duplicate of a handle a C function
/// closed; `{LIB}` stands for the path of the lib.
const CLOSED: &str = "\
ရေတွက်စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

လုပ်ဆောင်ချက် counter_open သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_close သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် counter_open ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
ကိန်းရှင် တူ သည် စက် ဖြစ်ပါစေ။
counter_close ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
တူ ၏ count ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// A C function that takes a handle over closes it for each value that
/// shares it, so a later read stops the run.
#[test]
fn a_closed_handle_stops_the_run()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let path = dir.path().join("main.zaka");
    let text = CLOSED.replace("{LIB}", &lib.display().to_string().replace('\\', "/"));
    std::fs::write(&path, text).unwrap();
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("the read stops the run");
    assert_eq!(trap.kind, TrapKind::ClosedHandle, "{trap}");
    assert_eq!(out, "");
}

/// A program whose plain handle result is NULL: C gives NULL for a step of 0.
const NULL_HANDLE: &str = "\
ရေတွက်စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ပါဝင်သည်။

လုပ်ဆောင်ချက် counter_open သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် တစ်ခု ကို ပြန်ပေးမည်။

ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"before\" ဟု ပြသမည်။
ကိန်းရှင် စက် သည် counter_open ဟူသော လုပ်ဆောင်ချက်ကို ၀ ဖြင့် လုပ်ဆောင်သော ရလဒ် ဖြစ်ပါစေ။
စက် ၏ count ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// A plain handle result for which C gives NULL stops the run with the trap
/// of a promised value, in the VM and as an executable.
#[test]
fn null_for_a_plain_handle_result_stops_the_run_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let path = write_program(dir.path(), &lib, NULL_HANDLE);
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("NULL for a plain handle stops the run");
    assert_eq!(trap.kind, TrapKind::NullFromC, "{trap}");
    assert_eq!(out, "before\n");
    let ran = native_run(&cc, &path, &lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert_eq!(ran.status.code(), Some(1), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "before\n");
    assert!(
        stderr.contains("C gave NULL where its declaration promised a value"),
        "{stderr}"
    );
}

/// The kinds and the functions of the programs that C calls back with text,
/// structs and handles; `{LIB}` stands for the path of the lib.
const CALLED_BACK: &str = "\
အမှတ် ဟူသော ဒေတာအစုအဝေးတွင် a ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် b ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။
မျဉ်း ဟူသော ဒေတာအစုအဝေးတွင် from ဟူသော အမှတ် တစ်ခု၊ marks ဟူသော အကွက် ၂ ခုရှိသော ၃၂ ဘစ် ကိန်းပြည့် စာရင်း \
                           တစ်ခု နှင့် weight ဟူသော ဒဿမကိန်း တစ်ခု တို့ပါဝင်သည်။
ရေတွက်စက် ဟူသော ပြင်ပမှ လာသော ဒေတာအစုအဝေးတွင် count ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု နှင့် step ဟူသော ၃၂ ဘစ် \
                           ကိန်းပြည့် တစ်ခု တို့ပါဝင်သည်။

လုပ်ဆောင်ချက် counter_open သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
step ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး ရေတွက်စက် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_next သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_close သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
c ဟူသော ရေတွက်စက် တစ်ခု ကို အပိုင်အနေဖြင့် လက်ခံမည်။
လုပ်ဆောင်ချက် text_call သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် \
                           အမျိုးအစား) တစ်ခု ကို လည်းကောင်း null ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို \
                           လည်းကောင်း လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် plain_text_call သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (စာသား တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို \
                           လည်းကောင်း null ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ၃၂ \
                           ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် point_call သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (အမှတ် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို လက်ခံပြီး \
                           ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် line_call သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (မျဉ်း တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို \
                           လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် counter_call သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် အမျိုးအစား) တစ်ခု ကို \
                           လည်းကောင်း c ဟူသော ရေတွက်စက် တစ်ခု ကို လည်းကောင်း null ဟူသော ၃၂ ဘစ် \
                           ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
လုပ်ဆောင်ချက် opt_counter_call သည် \"{LIB}\" မှလာသော လုပ်ဆောင်ချက်ဖြစ်၍
f ဟူသော (ရေတွက်စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးသော လုပ်ဆောင်ချက် \
                           အမျိုးအစား) တစ်ခု ကို လည်းကောင်း c ဟူသော ရေတွက်စက် တစ်ခု ကို လည်းကောင်း null \
                           ဟူသော ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို လည်းကောင်း လက်ခံပြီး ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို \
                           ပြန်ပေးမည်။

လုပ်ဆောင်ချက် စာကြည့် သည်
s ဟူသော စာသား ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
s ကို ပြသမည်။
အကယ်၍ s ထဲတွင် ဒေတာ ရှိလျှင် ၁ ကို ပြန်ပေးမည်။
၀ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် စာသက်သက် သည်
s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
s ကို ပြသမည်။
၇ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ပေါင်း သည်
p ဟူသော အမှတ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
p ကို ပြသမည်။
(p ၏ a) + (p ၏ b) ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် မျဉ်းကြည့် သည်
l ဟူသော မျဉ်း တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
l ကို ပြသမည်။
(l ၏ from) ၏ b ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ဘူးကြည့် သည်
b ဟူသော ရေတွက်စက် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
b ၏ count ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

လုပ်ဆောင်ချက် ဘူးရှိ သည်
b ဟူသော ရေတွက်စက် ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ် တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
အကယ်၍ b ထဲတွင် ဒေတာ ရှိလျှင် (b ထဲမှ ဒေတာ) ၏ step ကို ပြန်ပေးမည်။
၀ - ၁ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။

";

/// A program that hands C each such function.
const CALLS_BACK: &str = "\
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
text_call(စာကြည့် ဟူသော လုပ်ဆောင်ချက်၊ ၀) ကို ပြသမည်။
text_call(စာကြည့် ဟူသော လုပ်ဆောင်ချက်၊ ၁) ကို ပြသမည်။
plain_text_call(စာသက်သက် ဟူသော လုပ်ဆောင်ချက်၊ ၀) ကို ပြသမည်။
point_call(ပေါင်း ဟူသော လုပ်ဆောင်ချက်) ကို ပြသမည်။
line_call(မျဉ်းကြည့် ဟူသော လုပ်ဆောင်ချက်) ကို ပြသမည်။
ကိန်းရှင် စက် သည် counter_open(၅) ဖြစ်ပါစေ။
counter_next(စက်) ကို ပြသမည်။
counter_call(ဘူးကြည့် ဟူသော လုပ်ဆောင်ချက်၊ စက်၊ ၀) ကို ပြသမည်။
opt_counter_call(ဘူးရှိ ဟူသော လုပ်ဆောင်ချက်၊ စက်၊ ၀) ကို ပြသမည်။
opt_counter_call(ဘူးရှိ ဟူသော လုပ်ဆောင်ချက်၊ စက်၊ ၁) ကို ပြသမည်။
ကိန်းရှင် ကြိမ် သည် ၀ ဖြစ်ပါစေ။
လုပ်ဆောင်ချက် ရေ သည် s ဟူသော စာသား တစ်ခု ကို လက်ခံပြီး
အောက်ပါတို့ကို လုပ်ဆောင်ကာ ၃၂ ဘစ် ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။
ကြိမ် ကို ကြိမ် + ၁ ဟု ထားမည်။
\"{s} {ကြိမ်}\" ဟု ပြသမည်။
၉ ကို ပြန်ပေးမည်။
လုပ်ဆောင်စရာများ ပြီးဆုံးပါပြီး။
plain_text_call(ရေ ဟူသော လုပ်ဆောင်ချက်၊ ၀) ကို ပြသမည်။
plain_text_call(ရေ ဟူသော လုပ်ဆောင်ချက်၊ ၀) ကို ပြသမည်။
counter_close ဟူသော လုပ်ဆောင်ချက်ကို စက် ဖြင့် လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// What the program prints.
const WANT_CALLED_BACK: &str = "ဒေတာ(\"hello\")\n၁\nအလွတ်\n၀\nplain\n၇\nအမှတ်(a: ၃၊ b: \
                                ၄)\n၇\nမျဉ်း(from: အမှတ်(a: ၁၊ b: ၂)၊ marks: [၃၊ ၄]၊ \
                                weight: ၀.၅)\n၂\n၅\n၅\n၅\n-၁\nplain ၁\n၉\nplain ၂\n၉\n";

/// Write a program that calls the lib into a folder, with the path of the
/// lib in it.
///
/// # Arguments
///
/// * `dir` - The folder.
/// * `lib` - The lib.
/// * `text` - The program, with `{LIB}` for the path of the lib.
///
/// # Returns
///
/// The path of the program.
fn write_program(dir: &Path, lib: &Path, text: &str) -> PathBuf
{
    let path = dir.join("main.zaka");
    let text = text.replace("{LIB}", &lib.display().to_string().replace('\\', "/"));
    std::fs::write(&path, text).unwrap();
    path
}

/// C calls functions of the program back with text, an optional of text, a
/// struct as a value, a handle and an optional of a handle, and a closure
/// with text; each prints the same in the VM and as an executable.
#[test]
fn callbacks_take_text_structs_and_handles_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let path = write_program(dir.path(), &lib, &format!("{CALLED_BACK}{CALLS_BACK}"));
    assert_eq!(run_in_vm(&path), WANT_CALLED_BACK, "in the VM");
    assert_eq!(run_native(&cc, &path, &lib), WANT_CALLED_BACK, "as an executable");
}

/// A program that has C pass NULL for a parameter of plain text.
const NULL_TEXT: &str = "\
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
plain_text_call(စာသက်သက် ဟူသော လုပ်ဆောင်ချက်၊ ၀) ကို ပြသမည်။
plain_text_call(စာသက်သက် ဟူသော လုပ်ဆောင်ချက်၊ ၁) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// NULL from C for a parameter of plain text stops the run with the same
/// trap in the VM and as an executable.
#[test]
fn null_for_plain_text_stops_the_run_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let path = write_program(dir.path(), &lib, &format!("{CALLED_BACK}{NULL_TEXT}"));
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("NULL for plain text stops the run");
    assert_eq!(trap.kind, TrapKind::NullFromC, "{trap}");
    assert_eq!(out, "plain\n၇\n");
    let ran = native_run(&cc, &path, &lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert_eq!(ran.status.code(), Some(1), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "plain\n၇\n");
    let line = "the program stopped: C gave NULL where its declaration promised a \
                value; declare the result or the parameter as an optional, \"T \
                ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်\", when C can give NULL\n";
    assert_eq!(stop_line(&stderr), line, "stderr: {stderr}");
}

/// A program that has C pass NULL for a parameter of a plain handle.
const NULL_HANDLE_ARG: &str = "\
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ကိန်းရှင် စက် သည် counter_open(၅) ဖြစ်ပါစေ။
counter_call(ဘူးကြည့် ဟူသော လုပ်ဆောင်ချက်၊ စက်၊ ၀) ကို ပြသမည်။
counter_call(ဘူးကြည့် ဟူသော လုပ်ဆောင်ချက်၊ စက်၊ ၁) ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// NULL from C for a parameter of a plain handle stops the run, as for plain
/// text: the parameter promised a handle, and an optional takes NULL.
#[test]
fn null_for_a_plain_handle_parameter_stops_the_run_on_both_targets()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = build_lib(&cc, dir.path());
    let text = format!("{CALLED_BACK}{NULL_HANDLE_ARG}");
    let path = write_program(dir.path(), &lib, &text);
    let (out, ran) = vm_run(&path);
    let trap = ran.expect_err("NULL for a plain handle stops the run");
    assert_eq!(trap.kind, TrapKind::NullFromC, "{trap}");
    assert_eq!(out, "၀\n");
    let ran = native_run(&cc, &path, &lib);
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert_eq!(ran.status.code(), Some(1), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "၀\n");
    assert!(
        stderr.contains("C gave NULL where its declaration promised a value"),
        "{stderr}"
    );
}
