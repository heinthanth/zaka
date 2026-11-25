//! Packages as `zakac run` reads them from the folder of the app.

use std::path::Path;
use std::process::Command;

/// A function that exports `name` and gives back its argument squared.
fn square(name: &str) -> String
{
    format!(
        "{name} ဟူသော အများသုံးနိုင်သော လုပ်ဆောင်ချက် သည်\nက ဟူသော ကိန်းပြည့် တစ်ခု ကို လက်ခံပြီး\nအောက်ပါတို့ကို \
         လုပ်ဆောင်ကာ ကိန်းပြည့် တစ်ခု ကို ပြန်ပေးမည်။\nက * က ကို ပြန်ပေးမည်။\nလုပ်ဆောင်စရာများ \
         ပြီးဆုံးပါပြီး။\n"
    )
}

/// Write each file under `root`, its folders too.
fn write_files(root: &Path, files: &[(&str, &str)])
{
    for (path, text) in files
    {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
}

#[test]
fn a_file_of_the_app_that_a_dep_reaches_is_read_once()
{
    let dir = tempfile::tempdir().unwrap();
    let x = format!("\"app/util\" ကို ပြန်လည် မျှဝေမည်။\n{}", square("နှစ်ထပ်"));
    let main = "\"util.zaka\" ကို ထည့်သွင်းမည်။\n\"a/x\" ကို ထည့်သွင်းမည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို \
                လုပ်ဆောင်မည်။\nထပ် ဟူသော လုပ်ဆောင်ချက်ကို ၃ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။\nပရိုဂရမ် \
                ပြီးဆုံးပါပြီး။\n";
    write_files(
        dir.path(),
        &[
            ("app/zakac.toml", "[dependencies]\na = { path = \"../a\" }\n"),
            ("app/main.zaka", main),
            ("app/util.zaka", &square("ထပ်")),
            ("a/zakac.toml", "[dependencies]\napp = { path = \"../app\" }\n"),
            ("a/x.zaka", &x),
        ],
    );
    // Run from the folder of the app, so the root is `main.zaka` and its folder
    // is empty, while the dep reaches `util.zaka` as `../app/util.zaka`. Read
    // twice, the file would bring `ထပ်` from two files, which is ZK0262.
    let out = Command::new(env!("CARGO_BIN_EXE_zakac"))
        .env("ZAKAC_NO_CONFIG", "1")
        .args(["run", "--vm", "main.zaka"])
        .current_dir(dir.path().join("app"))
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{stderr}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "၉\n");
}
