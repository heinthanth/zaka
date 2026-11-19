//! Calls of C functions from native code, through libffi: the code describes
//! the C kind of each value, and the runtime makes the call interface of each
//! call shape once. libffi places structs passed as values and the values
//! after a `...`, as each ABI wants them.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;

use libffi::middle::Cif;
use libffi::middle::Type;
use libffi::raw::ffi_call;
use zakac_rust_sugars::ret_if;

use crate::callbacks::ZkCbShape;
use crate::heap::free;

/// The C kind of a value, as native code describes it.
#[repr(C)]
pub struct ZkCKind
{
    /// The shape: 0 none, 1 u8, 2 i8, 3 u16, 4 i16, 5 u32, 6 i32, 7 u64,
    /// 8 i64, 9 f32, 10 f64, 11 an address, 12 a struct.
    pub tag: u32,
    /// The count of members of a struct.
    pub count: u32,
    /// The C kind of each member of a struct.
    pub members: *const *const ZkCKind,
}

/// The shape of a call of a C function: the C kind of its result and of each
/// value it gets.
#[repr(C)]
pub struct ZkFfiShape
{
    /// The C kind of the result.
    pub ret: *const ZkCKind,
    /// The C kind of each value.
    pub args: *const *const ZkCKind,
    /// The count of values.
    pub nargs: u32,
    /// The count of values before the `...`.
    pub nfixed: u32,
    /// 1 when the C function takes values after a `...`.
    pub variadic: u32,
}

/// The shape of a function kind C calls back, as native code describes it:
/// [`ZkCbShape`] with C kinds in place of libffi kinds.
#[repr(C)]
pub struct ZkCbNativeShape
{
    /// Read the arguments libffi gives, run the thunk, write the result.
    pub bridge: extern "C" fn(*mut c_void, *mut *mut c_void, *mut u8),
    /// The C kind of the result.
    pub ret: *const ZkCKind,
    /// The C kind of each parameter.
    pub args: *const *const ZkCKind,
    /// The count of parameters.
    pub nargs: u32,
    /// Whether C can call the code of a function of the kind with no bridge.
    pub as_is: i32,
}

/// A value the runtime keeps for the whole run, shared on its one thread.
struct Kept<T>(T);
// SAFETY: one thread runs the program; the values are made once and read.
unsafe impl<T> Send for Kept<T> {}

/// The call interface of each call shape, under its address.
static CIFS: Mutex<Vec<(usize, Kept<Cif>)>> = Mutex::new(Vec::new());
/// The shape of each function kind C calls back, under the address of its
/// native shape.
static SHAPES: Mutex<Option<HashMap<usize, Kept<*const ZkCbShape>>>> = Mutex::new(None);

/// The libffi kind of a C kind.
///
/// # Arguments
///
/// * `t` - The C kind.
///
/// # Returns
///
/// The libffi kind.
fn ffi_type(t: *const ZkCKind) -> Type
{
    // SAFETY: native code writes each C kind it names.
    let t = unsafe { &*t };
    match t.tag
    {
        1 => Type::u8(),
        2 => Type::i8(),
        3 => Type::u16(),
        4 => Type::i16(),
        5 => Type::u32(),
        6 => Type::i32(),
        7 => Type::u64(),
        8 => Type::i64(),
        9 => Type::f32(),
        10 => Type::f64(),
        11 => Type::pointer(),
        12 => Type::structure(members(t.members, t.count)),
        _ => Type::void(),
    }
}

/// The libffi kinds of a list of C kinds.
///
/// # Arguments
///
/// * `list` - The C kinds.
/// * `count` - Their count.
///
/// # Returns
///
/// The libffi kinds.
fn members(list: *const *const ZkCKind, count: u32) -> Vec<Type>
{
    ret_if!(count == 0 || list.is_null(), Vec::new());
    // SAFETY: native code writes `count` C kinds.
    let all = unsafe { std::slice::from_raw_parts(list, count as usize) };
    all.iter().map(|t| ffi_type(*t)).collect()
}

/// Call a C function: libffi reads each value at its address, and writes the
/// result where `ret` points, a whole word for a narrow integer.
///
/// # Arguments
///
/// * `code` - The C function.
/// * `shape` - The shape of the call.
/// * `args` - The address of each value.
/// * `ret` - Where the result goes: room for it, and for a word at least.
#[unsafe(export_name = stamped!("ffi_call"))]
pub extern "C" fn zk_ffi_call(
    code: *const c_void, shape: *const ZkFfiShape, args: *mut *mut c_void,
    ret: *mut c_void,
)
{
    let mut cifs = CIFS.lock().unwrap_or_else(|e| e.into_inner());
    let mark = shape as usize;
    if !cifs.iter().any(|(k, _)| *k == mark)
    {
        // SAFETY: native code writes the shape of each call.
        let s = unsafe { &*shape };
        let args = members(s.args, s.nargs);
        let ret = ffi_type(s.ret);
        let cif = match s.variadic
        {
            0 => Cif::new(args, ret),
            _ => Cif::new_variadic(args, s.nfixed as usize, ret),
        };
        cifs.push((mark, Kept(cif)));
    }
    let cif = cifs.iter().find(|(k, _)| *k == mark).map(|(_, c)| c.0.as_raw_ptr());
    drop(cifs);
    let Some(cif) = cif
    else
    {
        return;
    };
    // SAFETY: `code` is the C function the shape describes; the values lie
    // at their addresses, and `ret` has room for the result.
    unsafe {
        let f: unsafe extern "C" fn() = std::mem::transmute(code);
        ffi_call(cif, Some(f), ret, args);
    }
}

/// The shape a native shape stands for, made on its first use: the libffi
/// kinds made from its C kinds, kept for the whole run.
///
/// # Arguments
///
/// * `native` - The native shape.
///
/// # Returns
///
/// The shape.
#[unsafe(export_name = stamped!("cb_shape_of"))]
pub extern "C" fn zk_cb_shape_of(native: *const ZkCbNativeShape) -> *const ZkCbShape
{
    let mut shapes = SHAPES.lock().unwrap_or_else(|e| e.into_inner());
    let map = shapes.get_or_insert_with(HashMap::new);
    ret_if!(let Some(s) = map.get(&(native as usize)), s.0);
    // SAFETY: native code writes the shape of each function kind it hands C.
    let n = unsafe { &*native };
    let args: &'static mut Vec<Type> = Box::leak(Box::new(members(n.args, n.nargs)));
    let ret: &'static Type = Box::leak(Box::new(ffi_type(n.ret)));
    let raw: Vec<*mut libffi::raw::ffi_type> =
        args.iter().map(|t| t.as_raw_ptr()).collect();
    let raw: &'static mut [*mut libffi::raw::ffi_type] =
        Box::leak(raw.into_boxed_slice());
    #[rustfmt::skip]
    let shape = Box::leak(Box::new(ZkCbShape { bridge: n.bridge, ret: ret.as_raw_ptr().cast(),
        args: raw.as_mut_ptr().cast(), nargs: n.nargs, as_is: n.as_is }));
    map.insert(native as usize, Kept(shape as *const ZkCbShape));
    shape
}

/// Free a string C handed over, with the C allocator; the heap count of
/// `--max-heap` never saw it.
///
/// # Arguments
///
/// * `p` - The string.
#[unsafe(export_name = stamped!("c_free"))]
pub extern "C" fn zk_c_free(p: *mut c_void)
{
    // SAFETY: C handed over a block of its allocator.
    unsafe { free(p) }
}
