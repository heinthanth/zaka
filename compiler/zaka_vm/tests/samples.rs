mod common;

use std::path::Path;
use std::path::PathBuf;

use common::compile;
use common::run_module;
use zaka_vm::machine::RunOptions;

/// Sample groups that hold programs with errors on purpose: those that do
/// not compile and those that go wrong at run time.
const SKIPPED_GROUPS: &[&str] = &["diagnostics", "mistakes"];

/// The crate dir.
fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Run a source file and compare its output with an expected file;
/// `ZAKA_BLESS=1` rewrites the file.
fn check_run(source: &Path, expected: &Path)
{
    let text = std::fs::read_to_string(source).unwrap();
    let options = RunOptions::builder().max_steps(10_000_000).build();
    let (out, result) = run_module(&compile(&text), &options);
    let name = source.display();
    assert_eq!(result, Ok(()), "{name} trapped after:\n{out}");
    if std::env::var("ZAKA_BLESS").is_ok_and(|v| v == "1")
    {
        std::fs::create_dir_all(expected.parent().unwrap()).unwrap();
        std::fs::write(expected, &out).unwrap();
    }
    let want = std::fs::read_to_string(expected)
        .unwrap_or_else(|_| panic!("no expected output for {name}"));
    assert_eq!(out, want, "output differs for {name}");
}

/// The sample files of one group dir, sorted.
fn group_files(dir: &Path) -> Vec<PathBuf>
{
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|f| f.path())
        .filter(|p| p.extension().is_some_and(|e| e == "zaka"))
        .collect();
    files.sort();
    files
}

#[test]
fn repo_samples_print_their_expected_output()
{
    let samples = root().join("../../samples");
    let mut groups: Vec<PathBuf> = std::fs::read_dir(&samples)
        .unwrap()
        .flatten()
        .map(|g| g.path())
        .filter(|p| p.is_dir())
        .filter(|p| !SKIPPED_GROUPS.iter().any(|s| p.ends_with(s)))
        .collect();
    groups.sort();
    assert!(!groups.is_empty());
    let mut count = 0;
    for group in groups
    {
        let group_name = group.file_name().unwrap().to_owned();
        for file in group_files(&group)
        {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            let expected = root()
                .join("tests/expected")
                .join(&group_name)
                .join(format!("{stem}.out"));
            check_run(&file, &expected);
            count += 1;
        }
    }
    assert!(count >= 30, "found {count} samples");
}

#[test]
fn counted_loop_edges()
{
    let source = root().join("tests/inputs/counted_edges.zaka");
    check_run(&source, &root().join("tests/expected/counted_edges.out"));
}
