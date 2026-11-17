//! The JIT: the program is lowered into code in the `zakac` process, which
//! the runtime linked into `zakac` serves, then run at once.

use std::ffi::CString;
use std::ffi::c_char;
use std::sync::Arc;

use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_jit::JITBuilder;
use cranelift_jit::JITModule;
use zaka_host::Gate;
use zaka_host::Grants;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_host_errors::Error;
use zakac_ir_builder::ir::Module as IrModule;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_semcheck::foreign::is_lib_path;
use zakac_span::source_file::SourceFile;

use crate::emit::Lowered;
use crate::emit::emit;
use crate::emit::module_error;

/// The main function of a program the JIT made, as C declares `main`.
type MainFn = extern "C" fn(i32, *const *const c_char, *const *const c_char) -> i32;

/// A JIT module for the host, with each function of the runtime found under
/// its C name.
///
/// # Returns
///
/// The module, or the error when the host has no backend.
fn module(program: &IrModule) -> Result<JITModule, Error>
{
    let mut flags = settings::builder();
    flags.set("opt_level", "speed").map_err(module_error)?;
    let isa = cranelift_native::builder().map_err(module_error)?;
    let isa = isa.finish(settings::Flags::new(flags)).map_err(module_error)?;
    let mut jb = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    for (name, addr) in zaka_runtime::exports()
    {
        jb.symbol(*name, *addr);
    }
    for f in &program.foreign
    {
        do_if!(let Some(addr) = c_fn_addr(&f.lib, &f.name)?, _ = jb.symbol(f.name.clone(), addr));
    }
    Ok(JITModule::new(jb))
}

/// Lower a program and run it in this process.
///
/// # Arguments
///
/// * `program` - The IR, after the ownership check.
/// * `files` - The source files, in source map order.
/// * `options` - The choices of the build.
/// * `gate` - The rights of the run, and what asks for one it lacks.
/// * `args` - The arguments of the start block: the program name first.
///
/// # Returns
///
/// The exit status, or the error of the lowering.
pub fn run(
    program: &IrModule, files: &[Arc<SourceFile>], options: TargetCOptions, gate: Gate,
    args: &[String],
) -> Result<i32, Error>
{
    let (m, lowered) = lower(program, files, options, &Grants::default())?;
    let code = m.get_finalized_function(lowered.main);
    zaka_runtime::set_gate(gate);
    let argv = c_list(args.iter().cloned());
    let env = std::env::vars_os()
        .map(|(k, v)| format!("{}={}", k.to_string_lossy(), v.to_string_lossy()));
    let envp = c_list(env);
    // SAFETY: the code is the main function the lowering defined, with this
    // signature.
    let main_fn: MainFn = unsafe { std::mem::transmute::<*const u8, MainFn>(code) };
    let status = main_fn(args.len() as i32, argv.1.as_ptr(), envp.1.as_ptr());
    drop((argv, envp));
    Ok(status)
}

/// Lower a program into code and data of this process, set to run.
///
/// # Arguments
///
/// * `program` - The IR, after the ownership check.
/// * `files` - The source files, in source map order.
/// * `options` - The choices of the build.
/// * `grants` - The rights the table holds; a run through `zakac` hands its own
///   to the runtime.
///
/// # Returns
///
/// The module, its code and data final, and what the lowering gave.
pub fn lower(
    program: &IrModule, files: &[Arc<SourceFile>], options: TargetCOptions,
    grants: &Grants,
) -> Result<(JITModule, Lowered), Error>
{
    let mut m = module(program)?;
    let lowered = emit(&mut m, program, files, options, grants, "zk_main", false)?;
    m.finalize_definitions().map_err(module_error)?;
    Ok((m, lowered))
}

/// A NULL-ended list of C strings, as `main` gets its arguments.
///
/// # Arguments
///
/// * `items` - The texts.
///
/// # Returns
///
/// The strings, which own the octets, and the list of their addresses.
fn c_list(items: impl Iterator<Item = String>) -> (Vec<CString>, Vec<*const c_char>)
{
    let owned: Vec<CString> =
        items.map(|s| CString::new(s.replace('\0', "")).unwrap_or_default()).collect();
    let mut ptrs: Vec<*const c_char> = owned.iter().map(|s| s.as_ptr()).collect();
    ptrs.push(std::ptr::null());
    (owned, ptrs)
}

/// The libs a program names that each run links, whose functions the process
/// holds.
const LINKED: &[&str] = &["libc", "libm"];

/// The address of a C function of a lib, opened as the VM opens it: a path as
/// written, else the file name of the lib on this host; libc and libm are the
/// process's own, and the JIT finds their functions in it.
///
/// # Arguments
///
/// * `lib` - The lib, as the program writes it.
/// * `name` - The C name.
///
/// # Returns
///
/// The address, `None` for a function of the process, or the error of a lib
/// that does not open.
fn c_fn_addr(lib: &str, name: &str) -> Result<Option<*const u8>, Error>
{
    ret_if!(LINKED.contains(&lib) && !cfg!(windows), Ok(None));
    let opened = open_lib(lib).map_err(module_error)?;
    let found = unsafe { opened.get::<unsafe extern "C" fn()>(name.as_bytes()) };
    let addr = found.map(|s| *s as *const u8).map_err(module_error)?;
    // The lib remains open for the run, as the code holds its functions.
    std::mem::forget(opened);
    Ok(Some(addr))
}

/// Open a lib as the VM opens it.
///
/// # Arguments
///
/// * `lib` - The lib, as the program writes it.
///
/// # Returns
///
/// The lib, or the error.
fn open_lib(lib: &str) -> Result<libloading::Library, libloading::Error>
{
    // SAFETY: opening a lib runs its initializers, as a C program's link does.
    ret_if!(is_lib_path(lib), unsafe { libloading::Library::new(lib) });
    let stem = lib.strip_prefix("lib").unwrap_or(lib);
    let windows_c = cfg!(windows) && matches!(stem, "c" | "m");
    let file = if windows_c
    {
        "ucrtbase.dll".into()
    }
    else
    {
        libloading::library_filename(stem)
    };
    // SAFETY: as above.
    let first = unsafe { libloading::Library::new(&file) };
    ret_if!(first.is_ok() || !cfg!(target_os = "linux"), first);
    let versioned = format!("{}.6", file.to_string_lossy());
    // SAFETY: as above.
    unsafe { libloading::Library::new(versioned) }
}
