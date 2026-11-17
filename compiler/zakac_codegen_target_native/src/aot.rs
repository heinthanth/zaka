//! AOT: the program as an object file for the ISA of its target, which the
//! C toolchain links with the runtime lib into an executable.

use std::sync::Arc;

use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_object::ObjectBuilder;
use cranelift_object::ObjectModule;
use zaka_host::Grants;
use zakac_ast::target::Target;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_host_errors::Error;
use zakac_ir_builder::ir::Module as IrModule;
use zakac_span::source_file::SourceFile;

use crate::emit::emit;
use crate::emit::module_error;

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
    let rest = match target.os
    {
        TargetOs::Macos => "apple-darwin",
        TargetOs::Linux => "unknown-linux-gnu",
        TargetOs::Windows => "pc-windows-gnu",
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
///
/// # Returns
///
/// The object, or the error of the lowering.
pub fn object(
    program: &IrModule, files: &[Arc<SourceFile>], options: TargetCOptions,
    grants: &Grants, target: &Target, want_clif: bool,
) -> Result<Object, Error>
{
    let mut flags = settings::builder();
    flags.set("opt_level", "speed").map_err(module_error)?;
    flags.set("is_pic", "true").map_err(module_error)?;
    let isa = cranelift_codegen::isa::lookup_by_name(&triple(target))
        .map_err(module_error)?;
    let isa = isa.finish(settings::Flags::new(flags)).map_err(module_error)?;
    let builder =
        ObjectBuilder::new(isa, "zaka", cranelift_module::default_libcall_names());
    let mut m = ObjectModule::new(builder.map_err(module_error)?);
    let lowered = emit(&mut m, program, files, options, grants, "main", want_clif)?;
    let octets = m.finish().emit().map_err(module_error)?;
    Ok(Object { octets, clif: lowered.clif })
}
