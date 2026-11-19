//! The table a program hands to `zk_start`, in the form
//! `zaka_target_c_runtime.h` gives it, and the globals that the inline code of
//! the header reads.

use std::ffi::CStr;
use std::ffi::c_char;
use std::ptr::null;

use zakac_rust_sugars::ret_if;

/// What a program holds of one right: each path or name, or a list.
#[repr(C)]
pub struct ZkGrant
{
    /// 1 when the right covers each path or name.
    pub all: i32,
    /// The count of listed paths or names.
    pub count: i32,
    /// The listed paths or names.
    pub items: *const *const c_char,
}

/// A place in the source: the index of its file, its line and its column.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZkSite
{
    /// The index of the file in the file names.
    pub file: u32,
    /// The line, from 1.
    pub line: u32,
    /// The column, from 1.
    pub col: u32,
}

/// One call in progress: the number of its function (0 for main, n + 1 for
/// row n of the function names) and the site of the statement it runs.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ZkFrame
{
    /// The number of the function.
    pub func: u32,
    /// The site, 0 for none.
    pub site: u32,
}

/// A function of the program and its name.
#[repr(C)]
pub struct ZkFnName
{
    /// The code of the function.
    pub code: crate::heap::ZkCode,
    /// The name, as a print writes it.
    pub name: *const c_char,
}

/// What the runtime reads of a program: the fields of `struct zk_program`.
#[repr(C)]
pub struct ZkProgram
{
    /// The depth limit of the calls.
    pub max_depth: u32,
    /// Whether the build has debug info.
    pub debug_info: u32,
    /// The heap limit in octets, 0 for none.
    pub max_heap: u64,
    /// The run-time checks, a bit for each, in the order of `RunCheck`.
    pub checks: u32,
    /// The rights the build granted, one for each right.
    pub grants: *const ZkGrant,
    /// Whether the program can ask for a right it lacks.
    pub grant_prompt: i32,
    /// What each right covers, then the whole environment.
    pub permission_text: *const *const c_char,
    /// The flag that grants each right.
    pub flag_text: *const *const c_char,
    /// The question for a right.
    pub ask_text: *const c_char,
    /// The template of each trap.
    pub trap_text: *const *const c_char,
    /// The line around a trap text.
    pub stopped_text: *const c_char,
    /// The word for true.
    pub true_text: *const c_char,
    /// The word for false.
    pub false_text: *const c_char,
    /// The words for the null address.
    pub null_text: *const c_char,
    /// The place of a stop, with a site.
    pub where_text: *const c_char,
    /// The place of a stop, with no site.
    pub in_text: *const c_char,
    /// The head of a backtrace.
    pub backtrace_text: *const c_char,
    /// A call of a backtrace, with a site.
    pub call_at_text: *const c_char,
    /// A call of a backtrace, with no site.
    pub call_text: *const c_char,
    /// The count of times a call repeats.
    pub call_again_text: *const c_char,
    /// The count of calls left out.
    pub calls_more_text: *const c_char,
    /// The hint for the full backtrace.
    pub full_hint_text: *const c_char,
    /// The word of a good result.
    pub good_text: *const c_char,
    /// The word of a fault.
    pub fault_text: *const c_char,
    /// The word of a good result with no value.
    pub good_bare: *const c_char,
    /// The word of a fault with no value.
    pub fault_bare: *const c_char,
    /// The word of an optional that holds data.
    pub data_text: *const c_char,
    /// The word of an optional with no data.
    pub no_data_text: *const c_char,
    /// The text before the name of a function value.
    pub fn_lead: *const c_char,
    /// The text after the name of a function value.
    pub fn_tail: *const c_char,
    /// Each function of the program, then a NULL row.
    pub fn_names: *const ZkFnName,
    /// The count of functions.
    pub fn_count: usize,
    /// The site of each statement a stop can name; NULL with no debug info.
    pub sites: *const ZkSite,
    /// The name of each source file the sites name.
    pub site_files: *const *const c_char,
}

/// The bit of the depth check in [`ZkProgram::checks`].
pub(crate) const DEPTH_BIT: u32 = 1 << 5;
/// The bit of the bounds check.
pub(crate) const BOUNDS_BIT: u32 = 1 << 2;
/// The bit of the text check.
pub(crate) const TEXT_BIT: u32 = 1 << 3;
/// The bit of the null check.
pub(crate) const NULL_BIT: u32 = 1 << 6;
/// The bit of the overlap check.
pub(crate) const OVERLAP_BIT: u32 = 1 << 7;

/// The table of the running program, from `zk_start`; C reads it as `zk_prog`.
#[unsafe(export_name = stamped!("prog"))]
#[allow(non_upper_case_globals)]
pub static mut zk_prog: *const ZkProgram = null();

/// The table of the running program.
///
/// # Returns
///
/// The table; `zk_start` set it before the program ran.
pub fn prog() -> &'static ZkProgram
{
    // SAFETY: `zk_start` sets the table before each other runtime call, and
    // the table lives as long as the program.
    unsafe { &*zk_prog }
}

/// Whether a check is on in the build.
///
/// # Arguments
///
/// * `bit` - The bit of the check.
///
/// # Returns
///
/// `true` when it is.
pub fn check_on(bit: u32) -> bool { prog().checks & bit != 0 }

/// Whether the build keeps the calls in progress: for the depth check, or for
/// the place and the backtrace of a stop with debug info.
///
/// # Returns
///
/// `true` when it does.
pub fn tracks() -> bool { prog().debug_info != 0 || check_on(DEPTH_BIT) }

/// The octets of a C text; NULL reads as blank.
///
/// # Arguments
///
/// * `p` - The text, NUL at its end.
///
/// # Returns
///
/// The octets, without the NUL.
pub fn text<'a>(p: *const c_char) -> &'a [u8]
{
    ret_if!(p.is_null(), b"");
    // SAFETY: the compiler writes each text of the table with a NUL, and the
    // generated code hands NUL-ended texts.
    unsafe { CStr::from_ptr(p).to_bytes() }
}

/// Row `n` of a C list of texts.
///
/// # Arguments
///
/// * `list` - The list.
/// * `n` - The row.
///
/// # Returns
///
/// The octets of the text.
pub fn row<'a>(list: *const *const c_char, n: usize) -> &'a [u8]
{
    // SAFETY: the table holds each list with the rows its enum counts.
    text(unsafe { *list.add(n) })
}

/// A field of a struct: its name, its offset in octets and its kind.
#[repr(C)]
pub struct ZkField
{
    /// The name.
    pub name: *const c_char,
    /// The offset in octets.
    pub offset: usize,
    /// The kind.
    pub kind: *const ZkKind,
}

/// A member of an enumeration: its value as the value kind prints it, and the
/// text a print writes for it.
#[repr(C)]
pub struct ZkMember
{
    /// The value, as text.
    pub value: *const c_char,
    /// The text a print writes.
    pub shown: *const c_char,
}

/// A data kind, in the form of `struct zk_kind`.
#[repr(C)]
pub struct ZkKind
{
    /// The shape, an `enum zk_tag`.
    pub tag: i32,
    /// The size in octets.
    pub size: usize,
    /// The cell of a list or a plain list; the value kind of an enumeration.
    pub cell: *const ZkKind,
    /// The cell count, the field count or the member count.
    pub len: usize,
    /// The fields of a struct.
    pub fields: *const ZkField,
    /// The members of an enumeration.
    pub members: *const ZkMember,
    /// The name of a struct.
    pub name: *const c_char,
    /// Whether a value holds text, a plain list or a closure on the heap.
    pub holds_text: i32,
    /// Whether a value holds a borrow.
    pub holds_borrow: i32,
    /// Whether the kind is an owner pointer, or holds one.
    pub owns: i32,
    /// Whether a plain list is a box.
    pub boxed: i32,
}

impl ZkKind
{
    /// The fields of a struct kind.
    ///
    /// # Returns
    ///
    /// The fields; blank for a kind with none.
    pub fn field_list(&self) -> &[ZkField]
    {
        ret_if!(self.fields.is_null(), &[]);
        // SAFETY: the compiler writes `len` fields for a struct kind.
        unsafe { std::slice::from_raw_parts(self.fields, self.len) }
    }

    /// The field with a name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The field, or `None`.
    pub fn field(&self, name: &str) -> Option<&ZkField>
    {
        self.field_list().iter().find(|f| text(f.name) == name.as_bytes())
    }
}
