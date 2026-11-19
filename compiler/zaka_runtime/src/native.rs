//! What code from Cranelift calls where C code uses a macro or libm: the
//! frames of the calls in progress and the remainder of floats.

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::program::DEPTH_BIT;
use crate::program::ZkFrame;
use crate::program::check_on;
use crate::program::prog;
use crate::program::tracks;
use crate::program::zk_depth;
use crate::program::zk_frames;
use crate::traps::zk_too_deep;

/// Note a call as it starts: count it, stop the run past the depth limit, and
/// keep its function; a call past the limit with no depth check is counted
/// but not kept. The inline `zk_enter` of C does the same.
///
/// # Arguments
///
/// * `func` - The number of its function: 0 for main, n + 1 for function n.
#[unsafe(export_name = stamped!("enter_call"))]
pub extern "C" fn zk_enter_call(func: u32)
{
    ret_if!(!tracks());
    // SAFETY: one thread runs the program, and the frames hold the depth
    // limit plus main.
    unsafe {
        zk_depth += 1;
        let depth = zk_depth;
        let limit = prog().max_depth;
        do_if!(depth > limit && check_on(DEPTH_BIT), zk_too_deep());
        ret_if!(depth > limit);
        *zk_frames.add(depth as usize) = ZkFrame { func, site: 0 };
    }
}

/// Note a call as it returns.
#[unsafe(export_name = stamped!("leave_call"))]
pub extern "C" fn zk_leave_call()
{
    ret_if!(!tracks());
    // SAFETY: one thread runs the program.
    unsafe { zk_depth -= 1 };
}

/// Note the site of the statement that runs, in a build with debug info.
///
/// # Arguments
///
/// * `site` - The index of the site in the site table.
#[unsafe(export_name = stamped!("at_site"))]
pub extern "C" fn zk_at_site(site: u32)
{
    ret_if!(prog().debug_info == 0);
    // SAFETY: one thread runs the program, and the frames hold the depth
    // limit plus main.
    unsafe {
        let depth = zk_depth;
        ret_if!(depth > prog().max_depth || zk_frames.is_null());
        (*zk_frames.add(depth as usize)).site = site;
    }
}

/// The remainder of an f64 division, as C's `fmod`.
///
/// # Arguments
///
/// * `a` - The dividend.
/// * `b` - The divisor.
///
/// # Returns
///
/// The remainder, with the sign of `a`.
#[unsafe(export_name = stamped!("fmod"))]
pub extern "C" fn zk_fmod(a: f64, b: f64) -> f64 { a % b }

/// The remainder of an f32 division, as C's `fmodf`.
///
/// # Arguments
///
/// * `a` - The dividend.
/// * `b` - The divisor.
///
/// # Returns
///
/// The remainder, with the sign of `a`.
#[unsafe(export_name = stamped!("fmodf"))]
pub extern "C" fn zk_fmodf(a: f32, b: f32) -> f32 { a % b }
