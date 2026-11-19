//! Closures C calls back. C calls a bare code address, so each closure a C
//! call gets runs behind a libffi closure, which holds the record. A closure
//! is live while a C call that got it runs, and after that while C has leave
//! (`ခေါ်ဆိုခွင့် ပြုမည်`). The code remains in place after its last use, so a
//! late call from C stops the run instead of running freed code.

use std::cell::Cell;
use std::ffi::c_void;
use std::sync::Mutex;

use libffi::raw::ffi_abi_FFI_DEFAULT_ABI;
use libffi::raw::ffi_cif;
use libffi::raw::ffi_closure;
use libffi::raw::ffi_closure_alloc;
use libffi::raw::ffi_prep_cif;
use libffi::raw::ffi_prep_closure_loc;
use libffi::raw::ffi_status_FFI_OK;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::heap::WORD;
use crate::heap::ZkCode;
use crate::heap::ZkFn;
use crate::heap::zero;
use crate::heap::zk_fn_bound;
use crate::heap::zk_fn_code;
use crate::heap::zk_fn_dup;
use crate::heap::zk_fn_rec;
use crate::heap::zk_fn_release;
use crate::heap::zk_fn_size;
use crate::program::ZkKind;
use crate::seqs::at;
use crate::seqs::zk_to_heap;
use crate::traps::trap;
use crate::traps::zk_plain_trap;

/// The bridge and the libffi kinds of a function kind, as `zk_cb_shape` holds
/// them; the compiler writes one for each function kind C calls back.
#[repr(C)]
pub struct ZkCbShape
{
    /// Read the arguments libffi gives, run the thunk with the record, and
    /// write the result where libffi wants it.
    pub bridge: extern "C" fn(*mut c_void, *mut *mut c_void, *mut u8),
    /// The libffi kind of the result, an `ffi_type`.
    pub ret: *mut c_void,
    /// The libffi kind of each parameter, each an `ffi_type`.
    pub args: *mut *mut c_void,
    /// The count of parameters.
    pub nargs: u32,
    /// Whether C can call the code of a function of the kind with no bridge.
    pub as_is: i32,
}

/// A closure C can call: its libffi closure and what it reads.
struct Cb
{
    /// The octets of the record when it was first seen: two values with the
    /// same function and the same captures are one closure.
    seen: Vec<u8>,
    /// The record the thunk takes last.
    record: *mut u8,
    /// The C calls that run now and got the closure.
    calls: u32,
    /// Whether C has leave to call the closure after those calls.
    kept: bool,
    /// The bridge and the libffi kinds of the function kind.
    shape: *const ZkCbShape,
    /// The call interface libffi reads.
    cif: ffi_cif,
    /// The native address C calls.
    code: *mut c_void,
}

/// Each closure C got, kept for the whole run; the runtime runs on one thread.
static CBS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
/// The closures lent to the C calls that run now, the latest last.
static LENDS: Mutex<Vec<usize>> = Mutex::new(Vec::new());

thread_local! {
    /// Set on the thread that runs the program, at its first lend or leave.
    static HOME: Cell<bool> = const { Cell::new(false) };
}

/// A closure C got, from its address.
///
/// # Arguments
///
/// * `p` - The address.
///
/// # Returns
///
/// The closure.
fn cb<'a>(p: usize) -> &'a mut Cb
{
    // SAFETY: each closure is a leaked box that lives for the whole run.
    unsafe { &mut *(p as *mut Cb) }
}

/// The code libffi runs when C calls a closure: a call from another thread
/// ends the run, since the program runs on one thread; a call with no lend
/// and no leave stops it; else the bridge runs the thunk.
///
/// # Arguments
///
/// * `_cif` - The call interface.
/// * `ret` - Where the result goes.
/// * `args` - The address of each argument.
/// * `data` - The closure.
unsafe extern "C" fn run(
    _cif: *mut ffi_cif, ret: *mut c_void, args: *mut *mut c_void, data: *mut c_void,
)
{
    if !HOME.get()
    {
        eprintln!(
            "C called a function of the program from a thread other than the one that \
             runs it; the run ends here"
        );
        std::process::abort();
    }
    let c = cb(data as usize);
    do_if!(!c.kept && c.calls == 0, zk_plain_trap(trap::CLOSURE_ESCAPE));
    // SAFETY: the compiler writes a shape for each closure kind.
    (unsafe { &*c.shape }.bridge)(ret, args, c.record)
}

/// The closure C got with the octets of a record.
///
/// # Arguments
///
/// * `seen` - The octets of the record.
///
/// # Returns
///
/// The closure, or `None`.
fn find(seen: &[u8]) -> Option<usize>
{
    let list = CBS.lock().unwrap_or_else(|e| e.into_inner());
    list.iter().copied().find(|&p| cb(p).seen == seen)
}

/// The octets of the record of a function value.
///
/// # Arguments
///
/// * `f` - The function value.
///
/// # Returns
///
/// The octets.
fn record_octets<'a>(f: ZkFn) -> &'a [u8]
{
    let size = tern!(zk_fn_bound(f) != 0, zk_fn_size(f), 2 * WORD);
    // SAFETY: a record holds `size` octets.
    unsafe { std::slice::from_raw_parts(zk_fn_rec(f), size) }
}

/// Stop the run: the runtime has no room for a closure C can call.
fn no_room() -> ! { zk_plain_trap(trap::OUT_OF_MEM) }

/// The closure C calls for a function value, made on its first use: a
/// closure, or a function whose parameters cross through the bridge. A
/// function lives for the whole run, so C has leave to call it at each time.
///
/// # Arguments
///
/// * `f` - The function value.
/// * `shape` - The bridge and the libffi kinds of its kind.
///
/// # Returns
///
/// The closure.
fn cb_of(f: ZkFn, shape: *const ZkCbShape) -> usize
{
    HOME.set(true);
    let seen = record_octets(f);
    ret_if!(let Some(found) = find(seen), found);
    #[rustfmt::skip]
    let fresh = Box::new(Cb { seen: seen.to_vec(), record: zk_fn_rec(f), calls: 0,
        kept: zk_fn_bound(f) == 0, shape, cif: Default::default(), code: std::ptr::null_mut() });
    let c = Box::leak(fresh);
    // SAFETY: the compiler writes a shape for each closure kind.
    let s = unsafe { &*shape };
    // SAFETY: the shape holds `nargs` libffi kinds, and the closure lives for
    // the whole run.
    unsafe {
        let prep = ffi_prep_cif(
            &mut c.cif,
            ffi_abi_FFI_DEFAULT_ABI,
            s.nargs,
            s.ret.cast(),
            s.args.cast(),
        );
        let closure: *mut ffi_closure =
            ffi_closure_alloc(size_of::<ffi_closure>(), &mut c.code).cast();
        do_if!(prep != ffi_status_FFI_OK || closure.is_null(), no_room());
        let data = (c as *mut Cb).cast::<c_void>();
        let done = ffi_prep_closure_loc(closure, &mut c.cif, Some(run), data, c.code);
        do_if!(done != ffi_status_FFI_OK, no_room());
    }
    let p = c as *mut Cb as usize;
    CBS.lock().unwrap_or_else(|e| e.into_inner()).push(p);
    p
}

/// The address C calls for a function value given to a C call: the code of a
/// function whose parameters cross as C passes them; else the libffi closure,
/// which for a closure of the program is lent until the call ends.
///
/// # Arguments
///
/// * `f` - The function value, not null.
/// * `shape` - The bridge and the libffi kinds of its kind.
///
/// # Returns
///
/// The address.
#[unsafe(export_name = stamped!("cb_code"))]
pub extern "C" fn zk_cb_code(f: ZkFn, shape: *const ZkCbShape) -> ZkCode
{
    let bound = zk_fn_bound(f) != 0;
    // SAFETY: the compiler writes a shape for each closure kind.
    ret_if!(!bound && unsafe { (*shape).as_is } != 0, zk_fn_code(f));
    let c = cb(cb_of(f, shape));
    ret_if!(!bound, code_of(c.code));
    LENDS.lock().unwrap_or_else(|e| e.into_inner()).push(c as *mut Cb as usize);
    c.calls += 1;
    do_if!(!c.kept, c.record = zk_fn_rec(f));
    code_of(c.code)
}

/// The code of a libffi closure, as C calls it.
///
/// # Arguments
///
/// * `p` - The executable address libffi gave.
///
/// # Returns
///
/// The code.
fn code_of(p: *mut c_void) -> ZkCode
{
    // SAFETY: libffi made `p` a function of the shape of the closure, and C
    // casts the code to that shape before it calls it.
    unsafe { std::mem::transmute::<*mut c_void, ZkCode>(p) }
}

/// The count of lends before a C call, for `zk_cb_end` once it returns.
///
/// # Returns
///
/// The count.
#[unsafe(export_name = stamped!("cb_lent"))]
pub extern "C" fn zk_cb_lent() -> usize { LENDS.lock().map_or(0, |l| l.len()) }

/// End the lends of the closures a C call got, once it returns.
///
/// # Arguments
///
/// * `from` - The count of lends before the call.
#[unsafe(export_name = stamped!("cb_end"))]
pub extern "C" fn zk_cb_end(from: usize)
{
    let mut lends = LENDS.lock().unwrap_or_else(|e| e.into_inner());
    while lends.len() > from
    {
        let p = lends.pop().unwrap_or_default();
        cb(p).calls -= 1;
    }
}

/// Give C leave to call a closure after the call that gave it: the leave holds
/// a count of the record. A function, NULL and a closure with leave are left
/// alone.
///
/// # Arguments
///
/// * `f` - The function value.
/// * `shape` - The bridge and the libffi kinds of its kind.
#[unsafe(export_name = stamped!("cb_leave"))]
pub extern "C" fn zk_cb_leave(f: ZkFn, shape: *const ZkCbShape)
{
    ret_if!(f.is_null() || zk_fn_bound(f) == 0);
    let c = cb(cb_of(f, shape));
    ret_if!(c.kept);
    c.record = zk_fn_dup(f).cast_mut().cast();
    c.kept = true;
}

/// End the leave `zk_cb_leave` gave a closure, letting go of the count of its
/// record. A value with no leave is left alone.
///
/// # Arguments
///
/// * `f` - The function value.
#[unsafe(export_name = stamped!("cb_revoke"))]
pub extern "C" fn zk_cb_revoke(f: ZkFn)
{
    ret_if!(f.is_null() || zk_fn_bound(f) == 0);
    let Some(p) = find(record_octets(f))
    else
    {
        return;
    };
    let c = cb(p);
    ret_if!(!c.kept);
    c.kept = false;
    zk_fn_release(c.record.cast_const().cast());
}

/// Write a new handle around an address C passed to a function of the
/// program: a box whose one field holds the address. NULL stops the run.
///
/// # Arguments
///
/// * `out` - Where the handle goes.
/// * `addr` - Where the address C passed lies.
/// * `list` - The kind of the cells of the box, as a list of one cell.
#[unsafe(export_name = stamped!("cb_handle"))]
pub extern "C" fn zk_cb_handle(
    out: *mut u8, addr: *const *mut c_void, list: *const ZkKind,
)
{
    // SAFETY: the bridge hands the address of an argument.
    do_if!(unsafe { *addr }.is_null(), zk_plain_trap(trap::NULL_FROM_C));
    zk_to_heap(out, addr.cast(), list, 0)
}

/// Write an optional of a handle C passed to a function of the program: no
/// data for NULL, else a new handle around the address. The tag sits at
/// offset 0.
///
/// # Arguments
///
/// * `out` - Where the optional goes: `size` octets.
/// * `size` - The size of the optional.
/// * `ok_at` - The offset of the handle in it.
/// * `addr` - Where the address C passed lies.
/// * `list` - The kind of the cells of the box, as a list of one cell.
#[unsafe(export_name = stamped!("cb_opt_handle"))]
pub extern "C" fn zk_cb_opt_handle(
    out: *mut u8, size: usize, ok_at: usize, addr: *const *mut c_void,
    list: *const ZkKind,
)
{
    zero(out, size);
    // SAFETY: the bridge hands the address of an argument.
    ret_if!(unsafe { *addr }.is_null());
    // SAFETY: the tag is the first octet of the optional.
    unsafe { *out = 1 };
    zk_cb_handle(at(out, ok_at), addr, list)
}

/// The count of closures lent now.
///
/// # Returns
///
/// The count.
pub(crate) fn lends() -> usize { LENDS.lock().unwrap_or_else(|e| e.into_inner()).len() }

/// End each lend past the first `n`: a trap inside a call of a C lib that lent
/// closures left them lent.
///
/// # Arguments
///
/// * `n` - The count of lends that remain.
pub(crate) fn end_lends_from(n: usize)
{
    LENDS.lock().unwrap_or_else(|e| e.into_inner()).truncate(n);
}
