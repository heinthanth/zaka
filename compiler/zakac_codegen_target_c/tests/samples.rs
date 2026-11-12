mod common;

use std::path::Path;
use std::path::PathBuf;

use common::c_files;
use common::cc;
use common::compile_and_run;
use common::root;
use common::samples;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_host_errors::Error;
use zakac_rust_sugars::do_if;

/// Sample groups that hold programs with errors on purpose: those that do
/// not compile and those that go wrong at run time.
const SKIPPED_GROUPS: &[&str] = &["diagnostics", "mistakes"];

/// The samples the C target does not run, each with the reason.
const NATIVE_SKIPPED: &[(&str, &str)] =
    &[("programs/start", "it prints its own path, argument 0, which differs per host")];

/// The modes each sample builds in: the default, and the fastest. A sound
/// program that never overflows prints the same in both.
const MODES: &[(OverflowMode, Checks)] =
    &[(OverflowMode::Stop, Checks::Default), (OverflowMode::Wrap, Checks::None)];

/// The top-level sample files of each group, sorted; the files a sample
/// imports sit in a folder below.
///
/// # Returns
///
/// The group name and the path of each sample.
fn sample_files() -> Vec<(String, PathBuf)>
{
    let mut groups: Vec<PathBuf> = std::fs::read_dir(samples())
        .unwrap()
        .flatten()
        .map(|g| g.path())
        .filter(|p| p.is_dir() && !SKIPPED_GROUPS.iter().any(|s| p.ends_with(s)))
        .collect();
    groups.sort();
    let files_of = |dir: &Path| {
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|f| f.path())
            .filter(|p| p.extension().is_some_and(|e| e == "zaka"))
            .collect();
        files.sort();
        let group = dir.file_name().unwrap().to_string_lossy().to_string();
        files.into_iter().map(move |f| (group.clone(), f))
    };
    groups.iter().flat_map(|g| files_of(g)).collect()
}

/// Build a sample in each mode and compare what it prints.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `file` - The sample.
/// * `want` - The output the VM prints.
/// * `name` - The name of the sample in messages.
///
/// # Returns
///
/// `None` when it built and printed `want`, or the construct the C
/// target cannot build at present.
fn check_sample(cc: &Tool, file: &Path, want: &str, name: &str) -> Option<String>
{
    for &(overflow, checks) in MODES
    {
        let name = format!("{name} ({overflow}, {checks})");
        let files = match c_files(file, overflow, checks)
        {
            Ok(files) => files,
            Err(Error::NotForTargetC { what }) => return Some(what),
            Err(error) => panic!("{name}: {error}"),
        };
        let run = compile_and_run(cc, &files, checks);
        assert_eq!(run.status, Some(0), "{name} stopped:\n{}", run.stderr);
        assert_eq!(run.stdout, want, "output differs for {name}");
    }
    None
}

/// Each repo sample the C target builds prints, as an executable, the output
/// the VM prints for it.
#[test]
fn repo_samples_print_their_expected_output_when_built()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let mut built = 0;
    let mut not_built = Vec::new();
    for (group, file) in sample_files()
    {
        let stem = file.file_stem().unwrap().to_string_lossy().to_string();
        let skipped =
            NATIVE_SKIPPED.iter().any(|(s, _)| *s == format!("{group}/{stem}"));
        do_if!(skipped, continue);
        let expected = root()
            .join("../zaka_vm/tests/expected")
            .join(&group)
            .join(format!("{stem}.out"));
        let want = std::fs::read_to_string(&expected).unwrap();
        match check_sample(&cc, &file, &want, &format!("{group}/{stem}"))
        {
            Some(what) => not_built.push(format!("{group}/{stem}: {what}")),
            None => built += 1,
        }
    }
    let list = not_built.join("\n  ");
    assert!(not_built.is_empty(), "the C target cannot build:\n  {list}");
    assert!(built >= 70, "built {built} samples");
}
