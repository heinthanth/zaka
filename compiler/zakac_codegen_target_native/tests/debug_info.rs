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
    object_for(file, debug_info, &native_target())
}

/// The object of a file of `inputs/debug/` of the C target, for a target.
///
/// # Arguments
///
/// * `file` - The name of the file.
/// * `debug_info` - Whether the build keeps the debug info.
/// * `target` - The target.
///
/// # Returns
///
/// The octets of the object.
fn object_for(
    file: &str, debug_info: bool, target: &zakac_ast::target::Target,
) -> Vec<u8>
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
        target,
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

#[test]
fn an_object_for_linux_carries_the_script_of_gdb()
{
    use zakac_ast::target::TargetAbi;
    use zakac_ast::target::TargetOs;

    let linux = zakac_ast::target::Target {
        os: TargetOs::Linux,
        abi: TargetAbi::Gnu,
        ..native_target()
    };
    let octets = object_for("vars.zaka", true, &linux);
    let file = object::File::parse(&*octets).unwrap();
    let section = file.section_by_name(".debug_gdb_scripts").unwrap();
    let data = section.data().unwrap();
    assert_eq!(data.first(), Some(&4));
    let script = zakac_codegen_target_native::debug::GDB_SCRIPT;
    assert!(data.windows(script.len()).any(|w| w == script.as_bytes()));
    let release = object_for("vars.zaka", false, &linux);
    let release = object::File::parse(&*release).unwrap();
    assert!(release.section_by_name(".debug_gdb_scripts").is_none());
}

/// The names of the records of each kind in the `.debug$S` of an
/// object, and the kinds of its subsections.
///
/// # Arguments
///
/// * `octets` - The object.
///
/// # Returns
///
/// The kind and name of each record, and the kind of each subsection.
fn codeview_of(octets: &[u8]) -> (Vec<(u16, String)>, Vec<u32>)
{
    use ms_codeview::syms::SymData;
    use ms_codeview::syms::SymIter;

    let file = object::File::parse(octets).unwrap();
    let section = file.section_by_name(".debug$S").unwrap();
    let data = section.data().unwrap();
    assert_eq!(data[..4], 4u32.to_le_bytes());
    let word = |at: usize| u32::from_le_bytes(data[at..at + 4].try_into().unwrap());
    let (mut at, mut kinds, mut records) = (4, Vec::new(), Vec::new());
    while at + 8 <= data.len()
    {
        let (kind, len) = (word(at), word(at + 4) as usize);
        kinds.push(kind);
        let content = &data[at + 8..at + 8 + len];
        let recs = SymIter::new(content).filter(|_| kind == 0xF1);
        let named = recs.filter_map(|rec| {
            let name = match rec.parse().ok()?
            {
                SymData::Proc(p) => p.name.to_string(),
                SymData::RegRel(r) => r.name.to_string(),
                SymData::Data(d) => d.name.to_string(),
                _ => return None,
            };
            Some((rec.kind.0, name))
        });
        records.extend(named);
        at += (8 + len).next_multiple_of(4);
    }
    (records, kinds)
}

#[test]
fn an_object_for_msvc_carries_codeview_in_place_of_dwarf()
{
    use zakac_ast::target::TargetAbi;
    use zakac_ast::target::TargetArch;
    use zakac_ast::target::TargetOs;

    let msvc = zakac_ast::target::Target {
        os: TargetOs::Windows,
        arch: TargetArch::X86_64,
        abi: TargetAbi::Msvc,
        ..native_target()
    };
    let octets = object_for("vars.zaka", true, &msvc);
    let (records, kinds) = codeview_of(&octets);
    let has = |kind: u16, name: &str| {
        let want = (kind, name.to_string());
        assert!(records.contains(&want), "{want:?} in {records:?}");
    };
    // S_LPROC32, S_REGREL32 and S_LDATA32.
    has(0x110f, "တွက်");
    has(0x110f, "ပရိုဂရမ်");
    has(0x1111, "က");
    has(0x1111, "င");
    has(0x1111, "ကျော်");
    has(0x110c, "အကြိမ်");
    assert!(kinds.contains(&0xF2) && kinds.contains(&0xF4), "{kinds:x?}");
    let file = object::File::parse(&*octets).unwrap();
    assert!(file.section_by_name(".debug_info").is_none());
    assert!(file.section_by_name(".debug_gdb_scripts").is_none());
}
