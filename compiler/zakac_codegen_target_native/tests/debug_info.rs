//! The DWARF of the native code: a line table from the code of each
//! statement to its line of the source, unless the build is for release.

use cranelift_codegen::gimli;
use object::Object;
use object::ObjectSection;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::TargetCOptions;

use crate::common::c_inputs;
use crate::common::checked;
use crate::common::native_target;

/// The object of `inputs/debug/lines.zaka` of the C target.
///
/// # Arguments
///
/// * `debug_info` - Whether the build keeps the debug info.
///
/// # Returns
///
/// The octets of the object.
fn object_of(debug_info: bool) -> Vec<u8>
{
    let path = c_inputs().join("debug/lines.zaka");
    let (sm, program) = checked(&path, OverflowMode::Stop);
    let options = TargetCOptions::builder().debug_info(debug_info).build();
    let grants = zaka_host::Grants::all();
    let made = zakac_codegen_target_native::aot::object(
        &program,
        sm.files(),
        options,
        &grants,
        &native_target(),
        false,
        None,
    );
    made.unwrap().octets
}

/// The lines of each row of the line tables of an object.
///
/// # Arguments
///
/// * `octets` - The object.
///
/// # Returns
///
/// The lines.
fn lines_of(octets: &[u8]) -> Vec<u64>
{
    let file = object::File::parse(octets).unwrap();
    let data = |id: gimli::SectionId| {
        let section = file.section_by_name(id.name());
        let octets = section.and_then(|s| s.data().ok()).unwrap_or(&[]);
        Ok::<_, gimli::Error>(gimli::EndianSlice::new(octets, gimli::LittleEndian))
    };
    let dwarf = gimli::Dwarf::load(data).unwrap();
    let mut lines = Vec::new();
    let mut headers = dwarf.units();
    while let Some(header) = headers.next().unwrap()
    {
        let unit = dwarf.unit(header).unwrap();
        let Some(program) = unit.line_program.clone()
        else
        {
            continue;
        };
        let mut rows = program.rows();
        while let Some((_, row)) = rows.next_row().unwrap()
        {
            lines.extend(row.line().map(|l| l.get()));
        }
    }
    lines
}

#[test]
fn the_line_table_names_each_statement()
{
    let lines = lines_of(&object_of(true));
    assert!(lines.contains(&5) && lines.contains(&9), "{lines:?}");
    let release = object::File::parse(&*object_of(false))
        .map(|f| f.section_by_name(".debug_line").is_none());
    assert_eq!(release.ok(), Some(true));
}
