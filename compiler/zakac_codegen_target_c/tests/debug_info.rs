//! The debug info of the C: a `#line` before each statement names its line
//! of the source, unless the build is for release.

use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::TargetCOptions;

use crate::common::c_files_with;
use crate::common::root;

/// The C of the modules of `inputs/debug/lines.zaka`.
///
/// # Arguments
///
/// * `debug_info` - Whether the build keeps the debug info.
///
/// # Returns
///
/// The C text.
fn module_c(debug_info: bool) -> String
{
    let path = root().join("tests/inputs/debug/lines.zaka");
    let target = TargetCOptions::builder().debug_info(debug_info).build();
    let files = c_files_with(&path, OverflowMode::Stop, target).unwrap();
    let modules =
        files.iter().filter(|f| f.name.starts_with('m') && f.name.ends_with(".c"));
    modules.map(|f| f.text.as_str()).collect()
}

#[test]
fn each_statement_names_its_line_of_the_source()
{
    let text = module_c(true);
    let lines: Vec<&str> =
        text.lines().map(str::trim).filter(|l| l.starts_with("#line")).collect();
    let names = |n: u32| lines.iter().any(|l| l.starts_with(&format!("#line {n} \"")));
    assert!(names(5) && names(9), "{lines:?}");
    assert!(lines.iter().all(|l| l.ends_with("/lines.zaka\"")), "{lines:?}");
    assert!(!module_c(false).contains("#line"));
}
