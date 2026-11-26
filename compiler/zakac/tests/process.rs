//! A program runs another program through the process module, as the right
//! to run grants it.

use std::process::Command;

/// What `zakac run` prints for a program that runs `zakac --version`.
///
/// # Arguments
///
/// * `flags` - The flags of the run.
///
/// # Returns
///
/// Its standard output.
fn run_version(flags: &[&str]) -> String
{
    let exe = env!("CARGO_BIN_EXE_zakac").replace('\\', "\\\\");
    let source = format!(
        "\"စံကား/လုပ်ငန်းစဉ်\" ကို ထည့်သွင်းမည်။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"{exe}\" ပရိုဂရမ်ကို [\"--version\"] ဖြင့် မောင်းနှင် သည့် ရလဒ် ကို ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
"
    );
    let dir =
        std::env::temp_dir().join(format!("zakac-process-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("version-{}.zaka", flags.join("").replace('=', "")));
    std::fs::write(&file, source).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zakac"));
    cmd.env("ZAKAC_NO_CONFIG", "1").arg("run").args(flags).arg(&file);
    let out = cmd.output().unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_run_with_the_right_gives_the_exit_code_and_the_output()
{
    for backend in ["--vm", "--jit"]
    {
        let out = run_version(&[backend, "--allow-run"]);
        assert!(out.contains("ပြီးဆုံးမှုကုဒ်: ၀"), "{backend}: {out}");
        assert!(out.contains("zakac v"), "{backend}: {out}");
    }
}

#[test]
fn a_run_without_the_right_fails_with_no_permission()
{
    for backend in ["--vm", "--jit"]
    {
        let out = run_version(&[backend, "--no-permission-prompt"]);
        assert!(out.contains("ခွင့်ပြုချက်မရှိခြင်း"), "{backend}: {out}");
    }
}
