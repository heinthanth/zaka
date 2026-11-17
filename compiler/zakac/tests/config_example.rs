//! The example config file at the root of the repo, `zakac.example.toml`, with
//! each of its settings turned on, reads as a config file of `zakac`.

use std::path::PathBuf;
use std::process::Command;

/// A program that prints one line.
const HELLO: &str = "\
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
\"မင်္ဂလာပါ\" ဟု ပြသမည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
";

/// The example file with the `#` taken off each line that sets a name, as a
/// reader turns a setting on.
fn each_setting_on() -> String
{
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../zakac.example.toml");
    let text = std::fs::read_to_string(root).expect("the example is readable");
    let on = |line: &str| {
        let rest = line.strip_prefix("# ").unwrap_or(line);
        let setting = rest.split_once(" = ").is_some_and(|(name, _)| {
            !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '-')
        });
        if setting
        {
            rest.to_string()
        }
        else
        {
            line.to_string()
        }
    };
    text.lines().map(on).collect::<Vec<_>>().join("\n")
}

#[test]
fn each_setting_of_the_example_reads_and_a_run_takes_its_own()
{
    let dir =
        std::env::temp_dir().join(format!("zakac-example-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("all.toml");
    std::fs::write(&config, each_setting_on()).unwrap();
    let file = dir.join("hello.zaka");
    std::fs::write(&file, HELLO).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_zakac"))
        .arg("--config")
        .arg(&config)
        .arg("run")
        .arg(&file)
        .current_dir(&dir)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "မင်္ဂလာပါ\n");
    std::fs::remove_dir_all(&dir).unwrap();
}
