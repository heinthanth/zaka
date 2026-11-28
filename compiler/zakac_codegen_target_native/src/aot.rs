//! AOT: the program as an object file for the ISA of its target, which the
//! C toolchain links with the runtime lib into an executable.

use std::sync::Arc;

use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_object::ObjectBuilder;
use cranelift_object::ObjectModule;
use zaka_host::Grants;
use zakac_ast::target::Target;
use zakac_ast::target::TargetAbi;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_codegen_target_c::lib_names::LibNames;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_host_errors::Error;
use zakac_ir_builder::ir::Module as IrModule;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::tern;
use zakac_span::source_file::SourceFile;

use crate::emit::emit;
use crate::emit::module_error;

/// The CPUs Cranelift builds for.
pub const ARCHES: [TargetArch; 3] =
    [TargetArch::X86_64, TargetArch::Aarch64, TargetArch::Riscv64];

/// The triple of a target, as Cranelift names it.
///
/// # Arguments
///
/// * `target` - The target; its CPU is one Cranelift builds for.
///
/// # Returns
///
/// The triple.
fn triple(target: &Target) -> String
{
    let arch = match target.arch
    {
        TargetArch::Aarch64 => "aarch64",
        TargetArch::Riscv64 => "riscv64gc",
        _ => "x86_64",
    };
    let rest = match (target.os, target.abi)
    {
        (TargetOs::Macos, _) => "apple-darwin",
        (TargetOs::Linux, _) => "unknown-linux-gnu",
        (TargetOs::Windows, TargetAbi::Msvc) => "pc-windows-msvc",
        (TargetOs::Windows, _) => "pc-windows-gnu",
        (TargetOs::Wasi, _) => "unknown-wasi",
    };
    format!("{arch}-{rest}")
}

/// What an AOT build makes: the object file, and the CLIF text when it was
/// asked for.
pub struct Object
{
    /// The octets of the object file.
    pub octets: Vec<u8>,
    /// The CLIF text of each function.
    pub clif: String,
    /// In a C lib, the index of the bridge of each function that code
    /// outside can call.
    pub export_shapes: Vec<usize>,
    /// Whether the program starts a thread, so the link takes the C guard of
    /// threads.
    pub threads: bool,
}

/// The Cranelift flags of both the JIT and AOT: code tuned for speed, and a
/// probe of each page of a frame larger than one page, inline. A Windows
/// OS thread commits its stack a page at a time behind a guard page, so a frame
/// that skips over the guard page faults; the probes cost a few stores on the
/// other hosts.
///
/// # Returns
///
/// The flags, or the error of a name Cranelift does not know.
pub(crate) fn base_flags() -> Result<settings::Builder, Error>
{
    let mut flags = settings::builder();
    flags.set("opt_level", "speed").map_err(module_error)?;
    flags.set("enable_probestack", "true").map_err(module_error)?;
    flags.set("probestack_strategy", "inline").map_err(module_error)?;
    Ok(flags)
}

/// Lower a program into an object file whose `main` starts it.
///
/// # Arguments
///
/// * `program` - The IR, after the ownership check.
/// * `files` - The source files, in source map order.
/// * `options` - The choices of the build.
/// * `grants` - The rights the executable holds.
/// * `target` - The target.
/// * `want_clif` - Whether to keep the CLIF text.
/// * `names` - The names of a C lib, whose names C never calls the object links
///   under.
///
/// # Returns
///
/// The object, or the error of the lowering.
pub fn object(
    program: &IrModule, files: &[Arc<SourceFile>], options: TargetCOptions,
    grants: &Grants, target: &Target, want_clif: bool, names: Option<&LibNames>,
) -> Result<Object, Error>
{
    let mut flags = base_flags()?;
    // COFF has no GOT for ARM64, so code for Windows on ARM64 reaches each
    // name it links to through an absolute address, which the linker
    // relocates.
    let arm_windows =
        target.os == TargetOs::Windows && target.arch == TargetArch::Aarch64;
    let pic = if arm_windows { "false" } else { "true" };
    flags.set("is_pic", pic).map_err(module_error)?;
    // The locals of the debug info count from the frame pointer.
    let frames = tern!(options.debug_info, "true", "false");
    flags.set("preserve_frame_pointers", frames).map_err(module_error)?;
    let isa = cranelift_codegen::isa::lookup_by_name(&triple(target))
        .map_err(module_error)?;
    let isa = isa.finish(settings::Flags::new(flags)).map_err(module_error)?;
    let builder =
        ObjectBuilder::new(isa, "zaka", cranelift_module::default_libcall_names());
    let mut m = ObjectModule::new(builder.map_err(module_error)?);
    let lowered = emit(&mut m, program, files, options, grants, "main", want_clif)?;
    let mut product = m.finish();
    do_if!(let Some(names) = names, rename(&mut product.object, names));
    let places = crate::debug::write(&mut product, &lowered.lines)?;
    crate::unwind::write(&mut product, &lowered.unwinds)?;
    let mut octets = product.emit().map_err(module_error)?;
    crate::debug::place_text(&mut octets, &places)?;
    #[rustfmt::skip]
    let object = Object { octets, clif: lowered.clif, export_shapes: lowered.export_shapes,
        threads: lowered.threads };
    Ok(object)
}

/// Give each name of a C lib that C never calls its linked form, as the C
/// files of the lib map it in `zaka_lib_names.h`.
///
/// # Arguments
///
/// * `object` - The object.
/// * `names` - The names of the lib.
fn rename(object: &mut cranelift_object::object::write::Object<'_>, names: &LibNames)
{
    let held = LibNames::held(|n| object.symbol_id(n.as_bytes()).is_some());
    let renames = held.iter().filter_map(|plain| {
        Some((object.symbol_id(plain.as_bytes())?, plain.len(), names.linked(plain)?))
    });
    for (id, plain_len, linked) in renames.collect::<Vec<_>>()
    {
        // The stored name carries the prefix of the format, such as `_`.
        let name = &mut object.symbol_mut(id).name;
        name.truncate(name.len() - plain_len);
        name.extend_from_slice(linked.as_bytes());
    }
}
