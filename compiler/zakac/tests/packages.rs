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

/// Whether git runs on this host.
fn has_git() -> bool
{
    Command::new("git").arg("--version").output().is_ok_and(|o| o.status.success())
}

/// Make a git repo of files with one commit, tagged `v1`.
fn git_repo(dir: &Path, files: &[(&str, &str)])
{
    write_files(dir, files);
    let git = |args: &[&str]| {
        let out = Command::new("git").args(args).current_dir(dir).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    };
    git(&["init", "--quiet"]);
    git(&["add", "-A"]);
    git(&[
        "-c",
        "user.name=t",
        "-c",
        "user.email=t@t",
        "commit",
        "--quiet",
        "-m",
        "one",
    ]);
    git(&["tag", "v1"]);
}

/// Run `zakac` in a folder with a cache folder of its own.
fn zakac(dir: &Path, cache: &Path, args: &[&str]) -> std::process::Output
{
    Command::new(env!("CARGO_BIN_EXE_zakac"))
        .env("ZAKAC_NO_CONFIG", "1")
        .env("ZAKAC_CACHE", cache)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

/// A dep with `git` and `rev`, as a manifest lists it.
fn git_dep(name: &str, repo: &Path) -> String
{
    let url = repo.display().to_string().replace('\\', "/");
    format!("[dependencies]\n{name} = {{ git = \"{url}\", rev = \"v1\" }}\n")
}

#[test]
fn fetch_pins_git_deps_and_their_deps_and_a_run_reads_them_offline()
{
    if !has_git()
    {
        eprintln!("git is not on the PATH: the test is skipped");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let [low, mid, app, cache] =
        ["low", "mid", "app", "cache"].map(|p| dir.path().join(p));
    git_repo(&low, &[("a.zaka", &square("ထပ်"))]);
    let b = format!("\"low/a\" ကို ပြန်လည် မျှဝေမည်။\n{}", square("နှစ်ထပ်"));
    git_repo(&mid, &[("zakac.toml", &git_dep("low", &low)), ("b.zaka", &b)]);
    let main = "\"mid/b\" ကို ထည့်သွင်းမည်။\nပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။\nထပ် ဟူသော လုပ်ဆောင်ချက်ကို \
                ၅ ဖြင့် လုပ်ဆောင်သော ရလဒ် ကို ပြသမည်။\nပရိုဂရမ် ပြီးဆုံးပါပြီး။\n";
    write_files(&app, &[("zakac.toml", &git_dep("mid", &mid)), ("main.zaka", main)]);
    let before = zakac(&app, &cache, &["run", "--vm", "main.zaka"]);
    assert!(String::from_utf8_lossy(&before.stderr).contains("ZK0271"));
    let fetched = zakac(&app, &cache, &["fetch"]);
    assert!(fetched.status.success(), "{}", String::from_utf8_lossy(&fetched.stderr));
    let lock = std::fs::read_to_string(app.join("zakac.lock")).unwrap();
    assert_eq!(lock.matches("[[git]]").count(), 2, "{lock}");
    // The repos go away: the run reads the checkouts in the cache alone.
    std::fs::rename(&low, dir.path().join("low-gone")).unwrap();
    std::fs::rename(&mid, dir.path().join("mid-gone")).unwrap();
    let ran = zakac(&app, &cache, &["run", "--vm", "main.zaka"]);
    assert!(ran.status.success(), "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "၂၅\n");
    let again = zakac(&app, &cache, &["fetch"]);
    assert!(again.status.success(), "{}", String::from_utf8_lossy(&again.stderr));
    assert_eq!(std::fs::read_to_string(app.join("zakac.lock")).unwrap(), lock);
}

#[test]
fn fetch_outside_a_package_is_an_error()
{
    let dir = tempfile::tempdir().unwrap();
    let out = zakac(dir.path(), &dir.path().join("cache"), &["fetch"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("zakac.toml"));
}
