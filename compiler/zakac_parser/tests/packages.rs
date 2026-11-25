use std::path::Path;
use std::sync::Arc;

use zakac_diagnostics::Code;
use zakac_host_errors::Error;
use zakac_lexer::lexer::LexerOptions;
use zakac_parser::load::Loaded;
use zakac_parser::load::load;
use zakac_span::file_loader::MemFileLoader;
use zakac_span::source_map::SourceMap;

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

/// Read `root` and its imports from disk.
fn load_disk(root: &Path) -> zakac_host_errors::Result<Loaded>
{
    let mut sm = SourceMap::new();
    let file = sm.load_file(root).unwrap();
    load(&file, &mut sm, &LexerOptions::default())
}

/// The diagnostic codes of a loaded program.
fn codes(loaded: &Loaded) -> Vec<Code>
{
    loaded.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn a_dep_reads_through_its_modules_table_and_its_paths()
{
    let dir = tempfile::tempdir().unwrap();
    let util = square("ထပ်");
    let circle = format!("\"geo/util.zaka\" မှ ထပ် ကို ထည့်သွင်းမည်။\n{}", square("နှစ်ထပ်"));
    write_files(
        dir.path(),
        &[
            (
                "app/zakac.toml",
                "[dependencies]\n\"ပုံသဏ္ဌာန်\" = { path = \"../shapes\" }\n",
            ),
            (
                "app/main.zaka",
                "\"ပုံသဏ္ဌာန်/စက်ဝိုင်း\" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။\n\"ပုံသဏ္ဌာန်/geo/util\" မှ ထပ် ကို \
                 ထည့်သွင်းမည်။\n",
            ),
            ("shapes/zakac.toml", "[package.modules]\n\"စက်ဝိုင်း\" = \"circle.zaka\"\n"),
            ("shapes/circle.zaka", &circle),
            ("shapes/geo/util.zaka", &util),
        ],
    );
    let loaded = load_disk(&dir.path().join("app/main.zaka")).unwrap();
    assert_eq!(codes(&loaded), []);
    let units = &loaded.program.units;
    let names: Vec<&str> = units.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names, ["ပုံသဏ္ဌာန်/geo/util.zaka", "ပုံသဏ္ဌာန်/circle.zaka", "main.zaka"]);
    let marks: Vec<bool> = units.iter().map(|u| u.package).collect();
    assert_eq!(marks, [true, true, false]);
}

#[test]
fn a_dep_reads_its_own_deps()
{
    let dir = tempfile::tempdir().unwrap();
    let mid = format!("\"low/a\" မှ ထပ် ကို ထည့်သွင်းမည်။\n{}", square("နှစ်ထပ်"));
    write_files(
        dir.path(),
        &[
            ("app/zakac.toml", "[dependencies]\nmid = { path = \"../mid\" }\n"),
            ("app/main.zaka", "\"mid/b\" မှ နှစ်ထပ် ကို ထည့်သွင်းမည်။\n"),
            ("mid/zakac.toml", "[dependencies]\nlow = { path = \"../low\" }\n"),
            ("mid/b.zaka", &mid),
            ("low/a.zaka", &square("ထပ်")),
        ],
    );
    let loaded = load_disk(&dir.path().join("app/main.zaka")).unwrap();
    assert_eq!(codes(&loaded), []);
    assert_eq!(loaded.program.units.len(), 3);
}

#[test]
fn an_unknown_package_is_zk0269_and_a_missing_module_zk0270()
{
    let dir = tempfile::tempdir().unwrap();
    write_files(
        dir.path(),
        &[
            ("app/zakac.toml", "[dependencies]\nshapes = { path = \"../shapes\" }\n"),
            (
                "app/main.zaka",
                "\"shapes/nope\" မှ က ကို ထည့်သွင်းမည်။\n\"shapez/x\" မှ က ကို ထည့်သွင်းမည်။\n",
            ),
            ("shapes/a.zaka", &square("က")),
        ],
    );
    let loaded = load_disk(&dir.path().join("app/main.zaka")).unwrap();
    assert_eq!(codes(&loaded), [Code::NoPackageModule, Code::UnknownPackage]);
}

#[test]
fn a_dep_whose_folder_is_not_there_is_a_host_error()
{
    let dir = tempfile::tempdir().unwrap();
    write_files(
        dir.path(),
        &[
            ("app/zakac.toml", "[dependencies]\ngone = { path = \"../gone\" }\n"),
            ("app/main.zaka", "\"gone/a\" မှ က ကို ထည့်သွင်းမည်။\n"),
        ],
    );
    let err = load_disk(&dir.path().join("app/main.zaka")).unwrap_err();
    assert!(matches!(err, Error::FileNotFound { .. }), "{err}");
}

#[test]
fn packages_that_import_each_other_are_zk0231()
{
    let dir = tempfile::tempdir().unwrap();
    write_files(
        dir.path(),
        &[
            ("app/zakac.toml", "[dependencies]\na = { path = \"../a\" }\n"),
            ("app/main.zaka", "\"a/x\" မှ က ကို ထည့်သွင်းမည်။\n"),
            ("a/zakac.toml", "[dependencies]\nb = { path = \"../b\" }\n"),
            ("a/x.zaka", &format!("\"b/z\" မှ ခ ကို ထည့်သွင်းမည်။\n{}", square("က"))),
            ("b/zakac.toml", "[dependencies]\na = { path = \"../a\" }\n"),
            ("b/z.zaka", &format!("\"a/x\" မှ က ကို ထည့်သွင်းမည်။\n{}", square("ခ"))),
        ],
    );
    let loaded = load_disk(&dir.path().join("app/main.zaka")).unwrap();
    assert_eq!(codes(&loaded), [Code::CircularImport]);
}

#[test]
fn packages_read_through_the_loader_of_the_source_map()
{
    let mut loader = MemFileLoader::new();
    let files = [
        (
            "/p/app/zakac.toml",
            "[dependencies]\nshapes = { path = \"../shapes\" }\n".to_string(),
        ),
        ("/p/app/main.zaka", "\"shapes/စက်ဝိုင်း\" မှ က ကို ထည့်သွင်းမည်။\n".to_string()),
        (
            "/p/shapes/zakac.toml",
            "[package.modules]\n\"စက်ဝိုင်း\" = \"circle.zaka\"\n".to_string(),
        ),
        ("/p/shapes/circle.zaka", square("က")),
    ];
    files.iter().for_each(|(path, text)| loader.insert(*path, text));
    let mut sm = SourceMap::with_loader(Arc::new(loader));
    let file = sm.load_file(Path::new("/p/app/main.zaka")).unwrap();
    let loaded = load(&file, &mut sm, &LexerOptions::default()).unwrap();
    assert_eq!(codes(&loaded), []);
    let names: Vec<&str> =
        loaded.program.units.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names, ["shapes/circle.zaka", "main.zaka"]);
}
