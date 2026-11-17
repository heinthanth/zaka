use std::path::Path;
use std::path::PathBuf;

use crate::common::root;
use crate::common::run_aot;
use crate::common::run_jit;
use crate::common::samples;

/// Sample groups that hold programs with errors on purpose.
const SKIPPED_GROUPS: &[&str] = &["diagnostics", "mistakes"];

/// The modes each sample runs in: overflow stops it, and overflow wraps; each
/// run holds each right, as the runs of the VM tests do.
const MODES: &[&[&str]] = &[&["--allow-all"], &["--allow-all", "--overflow", "wrap"]];

/// The samples the JIT does not run, each with the reason.
const SKIPPED: &[(&str, &str)] =
    &[("programs/start", "it prints its own path, argument 0, as the test passes it")];

/// The marker of the error of a construct the native target cannot build.
const NOT_BUILT: &str = "the native target cannot build ";

/// The top-level sample files of each group, sorted.
///
/// # Returns
///
/// The group name and the path of each sample.
pub fn sample_files() -> Vec<(String, PathBuf)>
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

/// Run one sample in each mode and compare what it prints with the output
/// of the VM.
///
/// # Arguments
///
/// * `group` - The group of the sample.
/// * `file` - The sample.
///
/// # Returns
///
/// `None` when it printed the expected output, or the sample and the
/// construct the native target cannot build at present.
fn check(group: &str, file: &Path) -> Option<String>
{
    let stem = file.file_stem().unwrap().to_string_lossy();
    let name = format!("{group}/{stem}");
    let expected = root()
        .join("../zaka_vm/tests/expected")
        .join(group)
        .join(format!("{stem}.out"));
    let want = std::fs::read_to_string(&expected).unwrap();
    for flags in MODES
    {
        let run = run_jit(file, flags);
        if let Some(at) = run.stderr.find(NOT_BUILT)
        {
            let what =
                run.stderr[at + NOT_BUILT.len()..].lines().next().unwrap_or_default();
            return Some(format!("{name}: {what}"));
        }
        assert_eq!(run.status, Some(0), "{name} {flags:?} stopped:\n{}", run.stderr);
        assert_eq!(run.stdout, want, "output differs for {name} {flags:?}");
    }
    // The same program, as an executable linked with the runtime lib.
    let overflow = zakac_borrow_checker::options::OverflowMode::Stop;
    let run = run_aot(file, overflow, &zaka_host::Grants::all())?;
    assert_eq!(run.status, Some(0), "{name} (AOT) stopped:\n{}", run.stderr);
    assert_eq!(run.stdout, want, "output differs for {name} (AOT)");
    None
}

/// Each repo sample the native target builds prints, through the JIT, the
/// output the VM prints for it. The samples run on each core at once.
#[test]
fn repo_samples_print_their_expected_output_through_the_jit()
{
    let listed = |(group, file): &(String, PathBuf)| {
        let name = format!("{group}/{}", file.file_stem().unwrap().to_string_lossy());
        !SKIPPED.iter().any(|(s, _)| *s == name)
    };
    let files: Vec<(String, PathBuf)> =
        sample_files().into_iter().filter(listed).collect();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let results: Vec<Option<String>> = std::thread::scope(|s| {
        let files = &files;
        let work = |i: usize| {
            let mine = files.iter().skip(i).step_by(threads);
            mine.map(|(group, file)| check(group, file)).collect::<Vec<_>>()
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
    assert!(
        not_built.is_empty(),
        "the native target cannot build ({built} built):\n  {list}"
    );
}
