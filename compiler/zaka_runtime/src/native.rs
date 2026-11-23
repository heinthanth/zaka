//! What code from Cranelift calls where C code uses a macro or libm: the
//! frames of the calls in progress and the remainder of floats.

use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::ctx::zk_ctx;
use crate::program::DEPTH_BIT;
use crate::program::ZkFrame;
use crate::program::check_on;
use crate::program::prog;
use crate::program::tracks;
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
    // SAFETY: the context of the thread lives as long as the thread, and its
    // frames hold the depth limit plus main.
    unsafe {
        let ctx = &mut *zk_ctx();
        ctx.depth += 1;
        let limit = prog().max_depth;
        do_if!(ctx.depth > limit && check_on(DEPTH_BIT), zk_too_deep());
        ret_if!(ctx.depth > limit);
        *ctx.frames.add(ctx.depth as usize) = ZkFrame { func, site: 0 };
    }
}

/// Note a call as it returns.
#[unsafe(export_name = stamped!("leave_call"))]
pub extern "C" fn zk_leave_call()
{
    ret_if!(!tracks());
    // SAFETY: the context of the thread lives as long as the thread.
    unsafe { (*zk_ctx()).depth -= 1 };
}

/// The site of no source region: a call of the runtime given it keeps the
/// site it runs under, which a trap inside it then names.
pub const NO_SITE: u32 = u32::MAX;

/// The site of a call of the runtime in progress, kept in the context of the
/// thread until the call ends, so a trap inside the call names the expression
/// that made it; the site before it comes back as the guard drops.
pub struct AtSite(Option<u32>);

/// Note the site of a call of the runtime for the traps inside it.
///
/// # Arguments
///
/// * `site` - The site of the expression that makes the call, or [`NO_SITE`].
///
/// # Returns
///
/// The guard that puts the site before it back.
pub fn at_site(site: u32) -> AtSite
{
    ret_if!(site == NO_SITE, AtSite(None));
    // SAFETY: the context of the thread lives as long as the thread.
    let ctx = unsafe { &mut *zk_ctx() };
    AtSite(Some(std::mem::replace(&mut ctx.call_site, site)))
}

impl Drop for AtSite
{
    fn drop(&mut self)
    {
        let Some(site) = self.0
        else
        {
            return;
        };
        // SAFETY: the context of the thread lives as long as the thread.
        unsafe { (*zk_ctx()).call_site = site };
    }
}

/// Note the site of the call of the runtime in progress in the frame of the
/// call that made it, as a trap stops the run, and clear it.
pub fn mark_call_site()
{
    // SAFETY: the context of the thread lives as long as the thread.
    let site = unsafe { std::mem::replace(&mut (*zk_ctx()).call_site, NO_SITE) };
    zk_at_site(site);
}

/// Note the site of the statement that runs, or of the expression whose
/// check failed, in a build with debug info; [`NO_SITE`] notes nothing.
///
/// # Arguments
///
/// * `site` - The index of the site in the site table.
#[unsafe(export_name = stamped!("at_site"))]
pub extern "C" fn zk_at_site(site: u32)
{
    ret_if!(prog().debug_info == 0 || site == NO_SITE);
    // SAFETY: the context of the thread lives as long as the thread, and its
    // frames hold the depth limit plus main.
    unsafe {
        let ctx = &mut *zk_ctx();
        ret_if!(ctx.depth > prog().max_depth || ctx.frames.is_null());
        (*ctx.frames.add(ctx.depth as usize)).site = site;
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
