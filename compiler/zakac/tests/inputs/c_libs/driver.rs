//! Calls the lib of mathlib.zaka from Rust: a sum, text in, a trap, then a
//! call after it.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe extern "C" {
    fn add_numbers(a: i32, b: i32, out: *mut i32) -> i32;
    fn greet(name: *const c_char, out: *mut u64) -> i32;
    fn divide(a: i64, b: i64, out: *mut i64) -> i32;
    fn zk_last_trap() -> *const c_char;
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
        println!("trap {}", CStr::from_ptr(zk_last_trap()).to_string_lossy());
        println!("after {}", add_numbers(1, 1, &mut sum));
    }
}
