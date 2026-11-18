//! What the C target does for WASI: the compiler of wasi-sdk, which builds a
//! WebAssembly module (preview 1) or component (preview 2) that a host such
//! as wasmtime runs.

use std::path::PathBuf;

use zakac_ast::target::TargetAbi;
use zakac_host_errors::Error;
use zakac_rust_sugars::ret_if;

use crate::toolchain::Tool;
use crate::toolchain::named;

/// The variable that names the folder of wasi-sdk.
pub const SDK_VAR: &str = "WASI_SDK_PATH";

/// The flags each WASI link adds: a stack of 8 MiB, as a native thread has,
/// where the linker would give 64 KiB. The user's link flags come after
/// them.
const STACK_FLAG: &str = "-Wl,-z,stack-size=8388608";

/// The flags a link of a WASI ABI adds: the stack, and for preview 2 the
/// export of `cabi_realloc`, which the component needs; the std of the
/// runtime lib defines it, and the linker keeps it unexported unless asked.
///
/// # Arguments
///
/// * `abi` - The ABI.
///
/// # Returns
///
/// The flags; none for an ABI that is not one of WASI.
pub fn link_flags(abi: TargetAbi) -> &'static [&'static str]
{
    match abi
    {
        TargetAbi::Wasip1 => &[STACK_FLAG],
        TargetAbi::Wasip2 => &[STACK_FLAG, "-Wl,--export=cabi_realloc"],
        _ => &[],
    }
}

/// The extension of a WASI program.
pub const EXTENSION: &str = "wasm";

/// Whether an ABI is one of WASI.
///
/// # Arguments
///
/// * `abi` - The ABI.
///
/// # Returns
///
/// `true` for preview 1 and preview 2.
pub fn is_wasi(abi: TargetAbi) -> bool
{
    matches!(abi, TargetAbi::Wasip1 | TargetAbi::Wasip2)
}

/// The target triple of a WASI ABI, as clang and Rust name it.
///
/// # Arguments
///
/// * `abi` - The ABI, one of WASI.
///
/// # Returns
///
/// The triple.
pub fn triple(abi: TargetAbi) -> &'static str
{
    match abi
    {
        TargetAbi::Wasip2 => "wasm32-wasip2",
        _ => "wasm32-wasip1",
    }
}

/// Find the C compiler of a WASI ABI: `--cc`, then the clang of
/// `$WASI_SDK_PATH` for the triple, then `<triple>-clang` on the PATH. The
/// `clang` of wasi-sdk named for a triple builds for it, with the sysroot of
/// the SDK. `CC` names a compiler of the host, so the search skips it.
///
/// # Arguments
///
/// * `chosen` - The compiler `--cc` names, words split on white space.
/// * `abi` - The ABI, one of WASI.
///
/// # Returns
///
/// The compiler, or the error when the one named is missing or none is
/// found.
pub fn find_cc(chosen: Option<&str>, abi: TargetAbi)
-> zakac_host_errors::Result<Tool>
{
    ret_if!(let Some(words) = chosen, named(words));
    let name = format!("{}-clang", triple(abi));
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let sdk =
        std::env::var_os(SDK_VAR).map(|dir| PathBuf::from(dir).join("bin").join(file));
    let in_sdk =
        sdk.filter(|p| p.is_file()).map(|p| Tool::builder().program(p).build());
    let on_path = || named(&name).ok();
    in_sdk.or_else(on_path).ok_or(Error::NoWasiCompiler { triple: triple(abi).into() })
}
