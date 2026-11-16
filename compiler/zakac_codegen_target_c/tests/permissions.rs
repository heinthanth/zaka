//! The rights a native executable holds are the ones its build granted: a
//! denied file or variable read fails with no permission, and a denied C
//! call stops the program and names the flag.

use zaka_host::Grant;
use zaka_host::Grants;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::Checks;
use zakac_codegen_target_c::options::TargetCOptions;

use crate::common::c_files_granted;
use crate::common::cc;
use crate::common::compile_and_run;
use crate::common::root;
use crate::common::stop_line;

/// The text a denied read prints.
const DENIED: &str = "အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ ခွင့်ပြုချက်မရှိခြင်း)\n";

/// The text a read of a file that is not there prints.
const MISSING: &str = "အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ မတွေ့ခြင်း)\n";

/// Build the permission program with some rights and run it.
fn run(grants: &Grants) -> Option<crate::common::Run>
{
    let cc = cc()?;
    let path = root().join("tests/inputs/permissions.zaka");
    let options = TargetCOptions::default();
    let files = c_files_granted(&path, OverflowMode::Stop, options, grants).unwrap();
    Some(compile_and_run(&cc, &files, Checks::Default))
}

/// With no right, the reads fail with no permission and the C call stops
/// the program.
#[test]
fn an_executable_with_no_right_is_denied()
{
    let Some(run) = run(&Grants::default())
    else
    {
        return;
    };
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, format!("{DENIED}{DENIED}"));
    let line = "the program stopped: calling the C function abs needs --allow-ffi\n";
    assert_eq!(stop_line(&run.stderr), line, "{}", run.stderr);
}

/// The rights the build granted let the reads and the call through.
#[test]
fn an_executable_with_the_rights_runs()
{
    let grants = Grants::builder()
        .read(Grant::All)
        .env(Grant::Some(vec!["NONE".into()]))
        .ffi(true)
        .build();
    let Some(run) = run(&grants)
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, format!("{DENIED}{MISSING}၄၂\n"));
}
