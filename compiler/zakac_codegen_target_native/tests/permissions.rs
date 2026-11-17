//! The rights of a run through the JIT are the ones `zakac run` grants, and
//! those of an executable the ones its build wrote into it: a denied file or
//! variable read fails with no permission, and a denied C call stops the
//! program and names the flag.

use zaka_host::Grant;
use zaka_host::Grants;
use zakac_borrow_checker::options::OverflowMode;

use crate::common::c_inputs;
use crate::common::run_aot;
use crate::common::run_jit;
use crate::common::stop_line;

/// The text a denied read prints.
const DENIED: &str = "အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ ခွင့်ပြုချက်မရှိခြင်း)\n";

/// The text a read of a file that is not there prints.
const MISSING: &str = "အမှားအယွင်း(အချက်အလက်အဝင်အထွက်အမှားအယွင်းများထဲမှ မတွေ့ခြင်း)\n";

/// With no right, the reads fail with no permission and the C call stops the
/// program.
#[test]
fn a_run_with_no_right_is_denied()
{
    let run = run_jit(&c_inputs().join("permissions.zaka"), &[]);
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, format!("{DENIED}{DENIED}"));
    let line = "the program stopped: calling the C function abs needs --allow-ffi\n";
    assert_eq!(stop_line(&run.stderr), line, "{}", run.stderr);
}

/// The rights `zakac run` grants let the reads and the call through.
#[test]
fn a_run_with_the_rights_runs()
{
    let flags = ["--allow-read", "--allow-env=NONE", "--allow-ffi"];
    let run = run_jit(&c_inputs().join("permissions.zaka"), &flags);
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, format!("{DENIED}{MISSING}၄၂\n"));
}

/// An executable built with no right denies the reads and stops at the C
/// call, as asking is off with no terminal.
#[test]
fn an_executable_with_no_right_is_denied()
{
    let Some(run) = run_aot(
        &c_inputs().join("permissions.zaka"),
        OverflowMode::Stop,
        &Grants::default(),
    )
    else
    {
        return;
    };
    assert_eq!(run.status, Some(1), "{}", run.stderr);
    assert_eq!(run.stdout, format!("{DENIED}{DENIED}"));
    let line = "the program stopped: calling the C function abs needs --allow-ffi\n";
    assert_eq!(stop_line(&run.stderr), line, "{}", run.stderr);
}

/// The rights written into an executable let the reads and the call through.
#[test]
fn an_executable_with_the_rights_runs()
{
    let names = Grant::Some(vec!["NONE".into()]);
    let grants = Grants::builder().read(Grant::All).env(names).ffi(true).build();
    let Some(run) =
        run_aot(&c_inputs().join("permissions.zaka"), OverflowMode::Stop, &grants)
    else
    {
        return;
    };
    assert_eq!(run.status, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, format!("{DENIED}{MISSING}၄၂\n"));
}
