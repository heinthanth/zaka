//! The CodeView of an object for the MSVC ABI. link.exe merges it into the
//! PDB of the executable, which Visual Studio, WinDbg and cdb read: a
//! procedure for each function under its Zaka name, its parameters and locals
//! at their offsets from the frame pointer, the globals, and the line of each
//! run of code. An object for the GNU ABI keeps DWARF ([`crate::debug`]).

use cranelift_object::object::Architecture;
use cranelift_object::object::RelocationEncoding;
use cranelift_object::object::RelocationFlags;
use cranelift_object::object::RelocationKind;
use cranelift_object::object::SectionKind;
use cranelift_object::object::write::Object;
use cranelift_object::object::write::Relocation;
use cranelift_object::object::write::SymbolId;
use ms_codeview::arch::amd64::Amd64Reg;
use ms_codeview::arch::arm64::Arm64Reg;
use ms_codeview::syms::SymKind;
use ms_codeview::syms::builder::SymBuilder;
use ms_codeview::types::TypeIndex;
use zakac_rust_sugars::do_for;
use zakac_semcheck::kind::DataKind;

use crate::debug::FnLines;
use crate::debug::FnVar;
use crate::debug::GlobalVar;
use crate::debug::LineTable;

/// The signature at the start of `.debug$S` and `.debug$T`: CodeView C13.
const SIGNATURE: u32 = 4;

/// The subsection of records: procedures, locals and globals.
const RECORDS: u32 = 0xF1;

/// The subsection of the lines of one function.
const LINES: u32 = 0xF2;

/// The subsection of the names of the source files.
const STRINGS: u32 = 0xF3;

/// The subsection of the source files, each pointing at its name.
const CHECKSUMS: u32 = 0xF4;

/// The size of each row of [`CHECKSUMS`]: the offset of the name, a checksum
/// size and kind of zero, and two octets that pad it.
const CHECKSUM_ROW: u32 = 8;

/// The mark of a line that starts a statement.
const STATEMENT: u32 = 1 << 31;

/// The flags of `S_FRAMEPROC`: the frame pointer is the base of the locals and
/// of the parameters.
const FRAME_BASE_FP: u32 = (2 << 14) | (2 << 16);

/// The language of `S_COMPILE3`: C, the nearest a debugger reads.
const LANGUAGE_C: u32 = 0;

/// A place of `.debug$S` the linker fills in: the offset of a name of the
/// object in its section (`SECREL`), or the section itself (`SECTION`).
#[derive(Debug, Clone, Copy)]
struct Place
{
    /// The offset of the place in the section.
    offset: u32,
    /// The name in the object.
    label: SymbolId,
    /// Whether the place takes the section, not the offset.
    index: bool,
}

/// What the records of the CodeView read: the line table, the name of each
/// function and global in the object, and the frame register of the CPU.
struct Program<'t>
{
    /// The line table.
    table: &'t LineTable,
    /// The name of each function in the object, in the order of the line
    /// table, then of each global.
    labels: &'t [SymbolId],
    /// The CodeView number of the frame pointer.
    frame: u16,
}

/// Add the CodeView of a program to an object for the MSVC ABI.
///
/// # Arguments
///
/// * `obj` - The object.
/// * `table` - The line table.
/// * `labels` - The name of each function in the object, in the order of the
///   line table, then of each global.
pub fn write(obj: &mut Object<'static>, table: &LineTable, labels: &[SymbolId])
{
    let arch = obj.architecture();
    let frame = frame_register(arch);
    let program = Program { table, labels, frame };
    let mut out = SIGNATURE.to_le_bytes().to_vec();
    let mut places = Vec::new();
    let (records, at) = records(&program, machine(arch));
    append(&mut out, RECORDS, &records, &at, &mut places);
    do_for!((n, f) in table.funcs.iter().enumerate(), {
        let (lines, at) = lines(table, f, labels[n]);
        append(&mut out, LINES, &lines, &at, &mut places);
    });
    let (checksums, names) = files(&table.files);
    append(&mut out, CHECKSUMS, &checksums, &[], &mut places);
    append(&mut out, STRINGS, &names, &[], &mut places);
    let section = obj.add_section(Vec::new(), b".debug$S".to_vec(), SectionKind::Debug);
    obj.append_section_data(section, &out, 4);
    do_for!(p in places, relocate(obj, section, p));
}

/// The CodeView number of the frame pointer of a CPU.
///
/// # Arguments
///
/// * `arch` - The CPU.
///
/// # Returns
///
/// `RBP` on x86-64, `FP` on ARM64.
fn frame_register(arch: Architecture) -> u16
{
    match arch
    {
        Architecture::Aarch64 => Arm64Reg::FP.0,
        _ => Amd64Reg::RBP.0,
    }
}

/// The machine of `S_COMPILE3`.
///
/// # Arguments
///
/// * `arch` - The CPU.
///
/// # Returns
///
/// `CV_CFL_ARM64` or `CV_CFL_AMD64`.
fn machine(arch: Architecture) -> u16
{
    match arch
    {
        Architecture::Aarch64 => 0xF6,
        _ => 0xD0,
    }
}

/// Append a subsection, padded to four octets, and its places.
///
/// # Arguments
///
/// * `out` - The section so far.
/// * `kind` - The kind of the subsection.
/// * `data` - Its content.
/// * `at` - Its places, from the start of its content.
/// * `places` - The places of the section.
fn append(
    out: &mut Vec<u8>, kind: u32, data: &[u8], at: &[Place], places: &mut Vec<Place>,
)
{
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    let base = out.len() as u32;
    out.extend_from_slice(data);
    out.resize(out.len().next_multiple_of(4), 0);
    places.extend(at.iter().map(|p| Place { offset: p.offset + base, ..*p }));
}

/// Add the relocation of one place.
///
/// # Arguments
///
/// * `obj` - The object.
/// * `section` - The section of the place.
/// * `place` - The place.
fn relocate(
    obj: &mut Object<'static>, section: cranelift_object::object::write::SectionId,
    place: Place,
)
{
    let (kind, size) = match place.index
    {
        true => (RelocationKind::SectionIndex, 16),
        false => (RelocationKind::SectionOffset, 32),
    };
    let flags =
        RelocationFlags::Generic { kind, encoding: RelocationEncoding::Generic, size };
    let reloc = Relocation {
        offset: u64::from(place.offset),
        symbol: place.label,
        addend: 0,
        flags,
    };
    obj.add_relocation(section, reloc).expect("a relocation of CodeView");
}

/// The records of a program: the object, the compiler, a procedure for
/// each function with its parameters and locals, and each global.
///
/// # Arguments
///
/// * `program` - The program.
/// * `machine` - The machine of `S_COMPILE3`.
///
/// # Returns
///
/// The records and their places.
fn records(program: &Program<'_>, machine: u16) -> (Vec<u8>, Vec<Place>)
{
    let mut b = SymBuilder::new();
    let mut places = Vec::new();
    let first = program.table.files.first().map_or("", String::as_str);
    {
        let mut r = b.record(SymKind::S_OBJNAME);
        r.enc.u32(0);
        r.enc.strz(first.into());
    }
    {
        let mut r = b.record(SymKind::S_COMPILE3);
        r.enc.u32(LANGUAGE_C);
        r.enc.u16(machine);
        do_for!(_ in 0..8, r.enc.u16(0));
        r.enc.strz("zakac".into());
    }
    do_for!((n, f) in program.table.funcs.iter().enumerate(), {
        procedure(&mut b, program, f, program.labels[n], &mut places);
    });
    let globals = program.table.globals.iter().enumerate();
    let named = globals.filter(|(_, g)| !g.name.starts_with('%'));
    do_for!((n, g) in named, {
        let label = program.labels[program.table.funcs.len() + n];
        global(&mut b, program.table.platform, g, label, &mut places);
    });
    (b.finish(), places)
}

/// The records of one function: `S_LPROC32`, `S_FRAMEPROC`, an `S_REGREL32`
/// for each parameter and local, and `S_END`.
///
/// # Arguments
///
/// * `b` - The records so far.
/// * `program` - The program.
/// * `f` - The function.
/// * `label` - Its name in the object.
/// * `places` - The places so far.
fn procedure(
    b: &mut SymBuilder, program: &Program<'_>, f: &FnLines, label: SymbolId,
    places: &mut Vec<Place>,
)
{
    {
        let mut r = b.record(SymKind::S_LPROC32);
        // The parent, the end and the next: the linker fills them in.
        do_for!(_ in 0..3, r.enc.u32(0));
        r.enc.u32(f.size);
        r.enc.u32(0);
        r.enc.u32(f.size);
        r.enc.u32(TypeIndex::T_NOTYPE.0);
        places.push(Place { offset: r.enc.len() as u32, label, index: false });
        r.enc.u32(0);
        places.push(Place { offset: r.enc.len() as u32, label, index: true });
        r.enc.u16(0);
        r.enc.u8(0);
        r.enc.strz(f.name.as_str().into());
    }
    {
        // Five sizes and offsets, the section of the handler, then the flags,
        // with no padding between the last two.
        let mut r = b.record(SymKind::S_FRAMEPROC);
        do_for!(_ in 0..5, r.enc.u32(0));
        r.enc.u16(0);
        r.enc.u32(FRAME_BASE_FP);
    }
    do_for!(v in &f.vars, local(b, program, v));
    b.record(SymKind::S_END);
}

/// The `S_REGREL32` of a parameter or a local: its offset from the frame
/// pointer, its kind and its name.
///
/// # Arguments
///
/// * `b` - The records so far.
/// * `program` - The program.
/// * `var` - The parameter or local.
fn local(b: &mut SymBuilder, program: &Program<'_>, var: &FnVar)
{
    let mut r = b.record(SymKind::S_REGREL32);
    r.enc.u32(var.at as i32 as u32);
    r.enc.u32(kind_index(var.kind, program.table.platform).0);
    r.enc.u16(program.frame);
    r.enc.strz(var.name.as_str().into());
}

/// The `S_LDATA32` of a global: its kind, its address and its name.
///
/// # Arguments
///
/// * `b` - The records so far.
/// * `platform` - Bit count of the platform-sized kinds.
/// * `g` - The global.
/// * `label` - Its name in the object.
/// * `places` - The places so far.
fn global(
    b: &mut SymBuilder, platform: u16, g: &GlobalVar, label: SymbolId,
    places: &mut Vec<Place>,
)
{
    let mut r = b.record(SymKind::S_LDATA32);
    r.enc.u32(kind_index(g.kind, platform).0);
    places.push(Place { offset: r.enc.len() as u32, label, index: false });
    r.enc.u32(0);
    places.push(Place { offset: r.enc.len() as u32, label, index: true });
    r.enc.u16(0);
    r.enc.strz(g.name.as_str().into());
}

/// The CodeView index of a data kind: a primitive for a number, a character,
/// a boolean or an address.
///
/// # Arguments
///
/// * `kind` - The kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The index, `T_NOTYPE` for each other kind.
fn kind_index(kind: DataKind, platform: u16) -> TypeIndex
{
    match kind
    {
        DataKind::Int(n) => int_index(n.signed, n.bits(platform)),
        DataKind::Float(32) => TypeIndex::T_REAL32,
        DataKind::Float(_) => TypeIndex::T_REAL64,
        DataKind::Char => TypeIndex::T_CHAR32,
        DataKind::Bool => TypeIndex::T_BOOL8,
        DataKind::Task(_) => TypeIndex::T_UINT8,
        DataKind::Ptr(_) | DataKind::Null | DataKind::Fn(_) => TypeIndex::T_64PVOID,
        _ => TypeIndex::T_NOTYPE,
    }
}

/// The CodeView index of an integer kind.
///
/// # Arguments
///
/// * `signed` - Whether it is signed.
/// * `bits` - Its width.
///
/// # Returns
///
/// The index.
fn int_index(signed: bool, bits: u16) -> TypeIndex
{
    match (signed, bits)
    {
        (true, 8) => TypeIndex::T_INT1,
        (false, 8) => TypeIndex::T_UINT1,
        (true, 16) => TypeIndex::T_INT2,
        (false, 16) => TypeIndex::T_UINT2,
        (true, 32) => TypeIndex::T_INT4,
        (false, 32) => TypeIndex::T_UINT4,
        (true, _) => TypeIndex::T_INT8,
        (false, _) => TypeIndex::T_UINT8,
    }
}

/// The lines of one function: its start and size, then a block of rows for
/// each run of rows in one source file.
///
/// # Arguments
///
/// * `table` - The line table.
/// * `f` - The function.
/// * `label` - Its name in the object.
///
/// # Returns
///
/// The subsection content and its places.
fn lines(table: &LineTable, f: &FnLines, label: SymbolId) -> (Vec<u8>, Vec<Place>)
{
    let places = vec![
        Place { offset: 0, label, index: false },
        Place { offset: 4, label, index: true },
    ];
    let mut out = Vec::new();
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&f.size.to_le_bytes());
    let site = |s: u32| table.sites.get(s as usize).copied();
    let rows: Vec<(u32, u32, u32)> = f
        .rows
        .iter()
        .filter_map(|(at, s)| site(*s).map(|(file, line, _)| (*at, file, line)))
        .filter(|(_, _, line)| *line > 0)
        .collect();
    do_for!(run in rows.chunk_by(|a, b| a.1 == b.1), block(&mut out, run));
    (out, places)
}

/// One block of rows in one source file.
///
/// # Arguments
///
/// * `out` - The subsection so far.
/// * `run` - The rows: the offset of the code, the file and the line.
fn block(out: &mut Vec<u8>, run: &[(u32, u32, u32)])
{
    let file = run[0].1 * CHECKSUM_ROW;
    let count = run.len() as u32;
    out.extend_from_slice(&file.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&(12 + 8 * count).to_le_bytes());
    do_for!((at, _, line) in run, {
        out.extend_from_slice(&at.to_le_bytes());
        out.extend_from_slice(&(line | STATEMENT).to_le_bytes());
    });
}

/// The source files: a row of [`CHECKSUMS`] for each, with no checksum, and
/// the string table of their paths, with `\` between their parts, since the
/// debuggers of Windows match a source file on that form alone.
///
/// # Arguments
///
/// * `paths` - The path of each file.
///
/// # Returns
///
/// The rows and the string table.
fn files(paths: &[String]) -> (Vec<u8>, Vec<u8>)
{
    let mut names = vec![0];
    let mut rows = Vec::new();
    do_for!(path in paths, {
        rows.extend_from_slice(&(names.len() as u32).to_le_bytes());
        rows.extend_from_slice(&[0, 0, 0, 0]);
        names.extend_from_slice(path.replace('/', "\\").as_bytes());
        names.push(0);
    });
    (rows, names)
}
