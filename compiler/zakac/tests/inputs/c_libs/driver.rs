//! Calls the lib of mathlib.zaka from Rust: a sum, text in, a trap, then a
//! call after it.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// One call in progress when a trap stopped the lib, as `zk_trap_frame`.
#[repr(C)]
#[allow(dead_code)]
struct Frame {
    function: *const c_char,
    file: *const c_char,
    line: u32,
    column: u32,
}

/// A trap, as `zk_trap`.
#[repr(C)]
#[allow(dead_code)]
struct Trap {
    code: i32,
    kind: *const c_char,
    message: *const c_char,
    lib: *const c_char,
    at: Frame,
    frames: *const Frame,
    frame_count: usize,
}

unsafe extern "C" {
    fn add_numbers(a: i32, b: i32, out: *mut i32) -> i32;
    fn greet(name: *const c_char, out: *mut u64) -> i32;
    fn divide(a: i64, b: i64, out: *mut i64) -> i32;
    fn zk_last_trap_from_mathlib() -> *const Trap;
}

/// A text of the lib.
fn text(p: *const c_char) -> String {
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

fn main() {
    unsafe {
        let mut sum = 0;
        println!("add {} {}", add_numbers(2, 40, &mut sum), sum);
        let name = CString::new("Rust").unwrap();
        let mut n = 0u64;
        println!("greet {} {}", greet(name.as_ptr(), &mut n), n);
        let mut q = 0i64;
        println!("divide {}", divide(7, 0, &mut q));
        let trap = &*zk_last_trap_from_mathlib();
        println!("trap {}", text(trap.message));
        println!("trap kind {} {}", text(trap.kind), text(trap.lib));
        println!("after {}", add_numbers(1, 1, &mut sum));
    }
}
