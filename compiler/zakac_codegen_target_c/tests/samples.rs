use std::path::Path;
use std::path::PathBuf;

use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::toolchain::Tool;
use zakac_host_errors::Error;

use crate::common::c_files;
use crate::common::cc;
use crate::common::compile_and_run;
use crate::common::root;
use crate::common::samples;

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

/// The name of a sample, `<group>/<stem>`.
///
/// # Arguments
///
/// * `group` - The group of the sample.
/// * `file` - The sample.
///
/// # Returns
///
/// The name.
fn sample_name(group: &str, file: &Path) -> String
{
    format!("{group}/{}", file.file_stem().unwrap().to_string_lossy())
}

/// Build one sample and compare what it prints with the output of the VM.
///
/// # Arguments
///
/// * `cc` - The C compiler.
/// * `group` - The group of the sample.
/// * `file` - The sample.
///
/// # Returns
///
/// `None` when it built and printed the expected output, or the sample with
/// the construct the C target cannot build at present.
fn check_listed(cc: &Tool, group: &str, file: &Path) -> Option<String>
{
    let name = sample_name(group, file);
    let stem = file.file_stem().unwrap().to_string_lossy();
    let expected = root()
        .join("../zaka_vm/tests/expected")
        .join(group)
        .join(format!("{stem}.out"));
    let want = std::fs::read_to_string(&expected).unwrap();
    check_sample(cc, file, &want, &name).map(|what| format!("{name}: {what}"))
}

/// Each repo sample the C target builds prints, as an executable, the output
/// the VM prints for it. The samples build on each core at once.
#[test]
fn repo_samples_print_their_expected_output_when_built()
{
    let Some(cc) = cc()
    else
    {
        return;
    };
    let listed = |(group, file): &(String, PathBuf)| {
        let name = sample_name(group, file);
        !NATIVE_SKIPPED.iter().any(|(s, _)| *s == name)
    };
    let files: Vec<(String, PathBuf)> =
        sample_files().into_iter().filter(listed).collect();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    // Thread `i` takes samples `i`, `i + threads`, ...; a failed assert in a
    // thread fails the test with its own message.
    let results: Vec<Option<String>> = std::thread::scope(|s| {
        let (cc, files) = (&cc, &files);
        let work = |i: usize| {
            let mine = files.iter().skip(i).step_by(threads);
            mine.map(|(group, file)| check_listed(cc, group, file)).collect::<Vec<_>>()
        };
        let workers: Vec<_> = (0..threads).map(|i| s.spawn(move || work(i))).collect();
        let joined = workers.into_iter().map(|w| w.join());
        joined
            .flat_map(|r| r.unwrap_or_else(|e| std::panic::resume_unwind(e)))
            .collect()
    });
    let built = results.iter().filter(|r| r.is_none()).count();
    let mut not_built: Vec<String> = results.into_iter().flatten().collect();
    not_built.sort();
    let list = not_built.join("\n  ");
    assert!(not_built.is_empty(), "the C target cannot build:\n  {list}");
    assert!(built >= 70, "built {built} samples");
}
