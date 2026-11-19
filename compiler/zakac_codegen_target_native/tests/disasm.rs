//! The listing of the machine code of an object, which the explorer shows:
//! each function under its Zaka name, for x86-64 and aarch64.

use zakac_ast::target::Target;
use zakac_ast::target::TargetAbi;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::TargetCOptions;
use zakac_codegen_target_native::disasm::listing;
use zakac_ir_builder::ir::shown_name;

use crate::common::c_inputs;
use crate::common::checked;
use crate::common::native_target;

/// The listing of `inputs/debug/lines.zaka` of the C target, built for a
/// CPU on Linux.
///
/// # Arguments
///
/// * `arch` - The CPU.
///
/// # Returns
///
/// The listing.
fn listing_for(arch: TargetArch) -> String
{
    let path = c_inputs().join("debug/lines.zaka");
    let (sm, program) = checked(&path, OverflowMode::Stop);
    let target =
        Target { os: TargetOs::Linux, arch, abi: TargetAbi::Gnu, ..native_target() };
    let options = TargetCOptions::builder().debug_info(false).build();
    let grants = zaka_host::Grants::all();
    let made = zakac_codegen_target_native::aot::object(
        &program,
        sm.files(),
        options,
        &grants,
        &target,
        false,
        None,
    );
    let octets = made.unwrap().octets;
    let name_of = |raw: &str| {
        let index = raw.strip_prefix("zk_f").and_then(|n| n.parse::<usize>().ok());
        let func = index.and_then(|n| program.funcs.get(n));
        func.map_or_else(|| raw.to_string(), |f| shown_name(&f.name).to_string())
    };
    listing(&octets, &name_of).unwrap()
}

#[test]
fn each_function_is_listed_under_its_name()
{
    for (arch, ret) in [(TargetArch::X86_64, "ret"), (TargetArch::Aarch64, "ret")]
    {
        let text = listing_for(arch);
        assert!(text.contains("စား:\n") && text.contains("main:\n"), "{text}");
        assert!(text.lines().any(|l| l.trim_end().ends_with(ret)), "{text}");
        assert!(text.contains("; zk_"), "the calls of the runtime are named:\n{text}");
    }
}
