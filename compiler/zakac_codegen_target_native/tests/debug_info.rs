//! The DWARF of the native code: a line table from the code of each
//! statement to its line of the source, and the parameters and locals of
//! each function with their kinds, unless the build is for release.

use cranelift_codegen::gimli;
use object::Object;
use object::ObjectSection;
use zakac_borrow_checker::options::OverflowMode;
use zakac_codegen_target_c::options::TargetCOptions;

use crate::common::c_inputs;
use crate::common::checked;
use crate::common::native_target;

/// The object of a file of `inputs/debug/` of the C target.
///
/// # Arguments
///
/// * `file` - The name of the file.
/// * `debug_info` - Whether the build keeps the debug info.
///
/// # Returns
///
/// The octets of the object.
fn object_of(file: &str, debug_info: bool) -> Vec<u8>
{
    let path = c_inputs().join("debug").join(file);
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
    let dwarf = dwarf_of(&file);
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
    let lines = lines_of(&object_of("lines.zaka", true));
    assert!(lines.contains(&5) && lines.contains(&9), "{lines:?}");
    let release = object::File::parse(&*object_of("lines.zaka", false))
        .map(|f| f.section_by_name(".debug_line").is_none());
    assert_eq!(release.ok(), Some(true));
}

/// The DWARF of an object.
type Dwarf<'o> = gimli::Dwarf<gimli::EndianSlice<'o, gimli::LittleEndian>>;

/// Read the DWARF of an object.
///
/// # Arguments
///
/// * `file` - The object.
///
/// # Returns
///
/// The DWARF.
fn dwarf_of<'o>(file: &object::File<'o>) -> Dwarf<'o>
{
    let data = |id: gimli::SectionId| {
        let section = file.section_by_name(id.name());
        let octets = section.and_then(|s| s.data().ok()).unwrap_or(&[]);
        Ok::<_, gimli::Error>(gimli::EndianSlice::new(octets, gimli::LittleEndian))
    };
    gimli::Dwarf::load(data).unwrap()
}

/// A parameter or a local the DWARF holds: whether it is a parameter, its
/// name, the name of its kind, and whether it has a place.
type Var = (bool, String, String, bool);

/// Each parameter and local of the DWARF of an object.
///
/// # Arguments
///
/// * `octets` - The object.
///
/// # Returns
///
/// The parameters and locals, in the order of the DWARF.
fn vars_of(octets: &[u8]) -> Vec<Var>
{
    let file = object::File::parse(octets).unwrap();
    let dwarf = dwarf_of(&file);
    let mut found = Vec::new();
    let mut headers = dwarf.units();
    while let Some(header) = headers.next().unwrap()
    {
        let unit = dwarf.unit(header).unwrap();
        let mut cursor = unit.entries();
        while let Some(die) = cursor.next_dfs().unwrap()
        {
            found.extend(var_of(&dwarf, &unit, die));
        }
    }
    found
}

/// The parameter or local of one DIE.
///
/// # Arguments
///
/// * `dwarf` - The DWARF.
/// * `unit` - The unit of the DIE.
/// * `die` - The DIE.
///
/// # Returns
///
/// The parameter or local, or `None` for a DIE of another tag.
fn var_of<'o>(
    dwarf: &Dwarf<'o>, unit: &gimli::Unit<gimli::EndianSlice<'o, gimli::LittleEndian>>,
    die: &gimli::DebuggingInformationEntry<gimli::EndianSlice<'o, gimli::LittleEndian>>,
) -> Option<Var>
{
    let param = die.tag() == gimli::DW_TAG_formal_parameter;
    let local = die.tag() == gimli::DW_TAG_variable;
    (param || local).then_some(())?;
    let text = |e: &gimli::DebuggingInformationEntry<_>| {
        let name = dwarf.attr_string(unit, e.attr_value(gimli::DW_AT_name)?).ok()?;
        Some(name.to_string_lossy().into_owned())
    };
    let gimli::AttributeValue::UnitRef(at) = die.attr_value(gimli::DW_AT_type)?
    else
    {
        return None;
    };
    let kind = text(&unit.entry(at).ok()?)?;
    let placed = die.attr_value(gimli::DW_AT_location).is_some();
    Some((param, text(die)?, kind, placed))
}

#[test]
fn each_parameter_and_local_has_its_kind_and_place()
{
    let vars = vars_of(&object_of("vars.zaka", true));
    let has = |param: bool, name: &str, kind: &str| {
        let want = (param, name.to_string(), kind.to_string(), true);
        assert!(vars.contains(&want), "{want:?} in {vars:?}");
    };
    has(true, "က", "isize");
    has(true, "သူ", "လူ");
    has(false, "ခ", "f64");
    has(false, "ဂ", "bool");
    has(false, "င", "text");
    // A local of the program start block, and a global.
    has(false, "ကျော်", "လူ");
    has(false, "အကြိမ်", "isize");
    // The names the compiler makes are left out.
    assert!(vars.iter().all(|(_, name, ..)| !name.starts_with('%')), "{vars:?}");
}

#[test]
fn a_release_build_keeps_no_local()
{
    assert!(vars_of(&object_of("vars.zaka", false)).is_empty());
}
