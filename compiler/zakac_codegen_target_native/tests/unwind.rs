//! The unwind tables of 64-bit Windows in an object of the native target:
//! `.xdata` with the unwind codes of each function, and `.pdata` with a row
//! for each: three image offsets on x64, two on ARM64.

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

/// The object of a program for Windows on a CPU.
///
/// # Arguments
///
/// * `arch` - The CPU.
///
/// # Returns
///
/// The octets of the object.
fn windows_object(arch: TargetArch) -> Vec<u8>
{
    let path = c_inputs().join("debug/lines.zaka");
    let (sm, program) = checked(&path, OverflowMode::Stop);
    let target =
        Target { os: TargetOs::Windows, arch, abi: TargetAbi::Gnu, ..native_target() };
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
    made.unwrap().octets
}

/// The count of the functions an object defines.
///
/// # Arguments
///
/// * `file` - The object.
///
/// # Returns
///
/// The count.
fn functions(file: &object::File) -> usize
{
    let defined =
        |s: &object::Symbol| s.kind() == SymbolKind::Text && s.is_definition();
    file.symbols().filter(defined).count()
}

#[test]
fn each_function_of_windows_code_has_a_row_of_pdata()
{
    let octets = windows_object(TargetArch::X86_64);
    let file = object::File::parse(&*octets).unwrap();
    let count = functions(&file);
    let pdata = file.section_by_name(".pdata").expect("a .pdata section");
    let xdata = file.section_by_name(".xdata").expect("a .xdata section");
    assert!(count > 0 && xdata.size() > 0);
    assert_eq!(pdata.size(), 12 * count as u64);
    assert_eq!(pdata.relocations().count(), 3 * count);
}

/// On ARM64 a row of `.pdata` holds the start of its function and the place
/// of its codes, and each part of `.xdata` starts with a header word whose
/// count of words of codes covers them, with `end` in the last word.
#[test]
fn each_function_of_windows_arm64_code_has_a_row_of_pdata()
{
    let octets = windows_object(TargetArch::Aarch64);
    let file = object::File::parse(&*octets).unwrap();
    let count = functions(&file);
    let pdata = file.section_by_name(".pdata").expect("a .pdata section");
    let xdata = file.section_by_name(".xdata").expect("a .xdata section");
    assert!(count > 0);
    assert_eq!(pdata.size(), 8 * count as u64);
    assert_eq!(pdata.relocations().count(), 2 * count);
    let data = xdata.data().unwrap();
    let mut at = 0;
    for _ in 0..count
    {
        let header = u32::from_le_bytes(data[at..at + 4].try_into().unwrap());
        let words = (header >> 27) as usize;
        assert!(header & 0x3ffff > 0 && words > 0, "{header:#x}");
        let codes = &data[at + 4..at + 4 + words * 4];
        assert_eq!(codes.last(), Some(&0xE4), "{codes:x?}");
        at += 4 + words * 4;
    }
    assert_eq!(at as u64, xdata.size());
}
