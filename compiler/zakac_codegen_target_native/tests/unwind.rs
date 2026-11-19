//! The unwind tables of 64-bit Windows in an object of the native target:
//! `.xdata` with the unwind codes of each function, and `.pdata` with a row of
//! three image offsets for each.

use object::Object;
use object::ObjectSection;
use object::ObjectSymbol;
use object::SymbolKind;
use zakac_ast::target::Target;
use zakac_ast::target::TargetAbi;
use zakac_ast::target::TargetArch;
use zakac_ast::target::TargetOs;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::TargetCOptions;

use crate::common::c_inputs;
use crate::common::checked;
use crate::common::native_target;

#[test]
fn each_function_of_windows_code_has_a_row_of_pdata()
{
    let path = c_inputs().join("debug/lines.zaka");
    let (sm, program) = checked(&path, OverflowMode::Stop);
    let target = Target {
        os: TargetOs::Windows,
        arch: TargetArch::X86_64,
        abi: TargetAbi::Gnu,
        ..native_target()
    };
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
    let file = object::File::parse(&*octets).unwrap();
    let functions =
        file.symbols().filter(|s| s.kind() == SymbolKind::Text && s.is_definition());
    let count = functions.count();
    let pdata = file.section_by_name(".pdata").expect("a .pdata section");
    let xdata = file.section_by_name(".xdata").expect("a .xdata section");
    assert!(count > 0 && xdata.size() > 0);
    assert_eq!(pdata.size(), 12 * count as u64);
    assert_eq!(pdata.relocations().count(), 3 * count);
}
