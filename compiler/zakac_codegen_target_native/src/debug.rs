//! The DWARF of an object: a line table from the code of each statement to
//! its line of the source, and a subprogram for each function with its name
//! in Zaka, so a debugger stops on a line of Zaka and names the functions of a
//! backtrace, with its parameters and locals and the globals of the program,
//! each with the DWARF of its kind ([`crate::dwarf_kinds`]).

use std::collections::HashMap;

use cranelift_codegen::gimli;
use cranelift_codegen::gimli::Encoding;
use cranelift_codegen::gimli::Format;
use cranelift_codegen::gimli::LineEncoding;
use cranelift_codegen::gimli::RunTimeEndian;
use cranelift_codegen::gimli::SectionId;
use cranelift_codegen::gimli::write::Address;
use cranelift_codegen::gimli::write::AttributeValue;
use cranelift_codegen::gimli::write::DwarfUnit;
use cranelift_codegen::gimli::write::EndianVec;
use cranelift_codegen::gimli::write::Expression;
use cranelift_codegen::gimli::write::LineProgram;
use cranelift_codegen::gimli::write::LineString;
use cranelift_codegen::gimli::write::Range;
use cranelift_codegen::gimli::write::RangeList;
use cranelift_codegen::gimli::write::Sections;
use cranelift_codegen::gimli::write::Writer;
use cranelift_module::DataId;
use cranelift_module::FuncId;
use cranelift_object::ObjectProduct;
use cranelift_object::object::Architecture;
use cranelift_object::object::BinaryFormat;
use cranelift_object::object::RelocationEncoding;
use cranelift_object::object::RelocationFlags;
use cranelift_object::object::RelocationKind;
use cranelift_object::object::SectionKind;
use cranelift_object::object::write::Relocation;
use cranelift_object::object::write::StandardSegment;
use zakac_host_errors::Error;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::dwarf_kinds::Kinds;
use crate::emit::module_error;

/// A parameter or a local of a function, as a debugger shows it.
#[derive(Debug, Clone, bon::Builder)]
pub struct FnVar
{
    /// Its name.
    #[builder(into)]
    pub name: String,
    /// Its kind.
    pub kind: DataKind,
    /// Whether it is a parameter.
    pub param: bool,
    /// The site of its declaration.
    pub site: Option<u32>,
    /// The offset of its stack slot from the frame pointer.
    pub at: i64,
}

/// The lines of one function: where each run of its code starts, with the
/// site of the statement the run belongs to.
#[derive(Debug, Clone, bon::Builder)]
pub struct FnLines
{
    /// The function.
    pub id: FuncId,
    /// Its name, as a backtrace shows it.
    #[builder(into)]
    pub name: String,
    /// The size of its code.
    pub size: u32,
    /// The offset of each run of code and its site, in the order of the code.
    pub rows: Vec<(u32, u32)>,
    /// Its parameters and locals that live in a stack slot.
    #[builder(default)]
    pub vars: Vec<FnVar>,
}

/// What the line table of a program reads: its functions, its sites, and the
/// path of each source file.
#[derive(Debug, Clone, Default)]
pub struct LineTable
{
    /// The functions with code.
    pub funcs: Vec<FnLines>,
    /// Each site: the file, the line and the column.
    pub sites: Vec<(u32, u32, u32)>,
    /// The path of each source file, as a debugger opens it.
    pub files: Vec<String>,
    /// Bit count of the platform-sized kinds.
    pub platform: u16,
    /// The globals the program names.
    pub globals: Vec<GlobalVar>,
}

/// A global of the program, as a debugger shows it.
#[derive(Debug, Clone, bon::Builder)]
pub struct GlobalVar
{
    /// Its name.
    #[builder(into)]
    pub name: String,
    /// Its kind.
    pub kind: DataKind,
    /// The site of its declaration.
    pub site: Option<u32>,
    /// Its storage.
    pub id: DataId,
}

/// A place of a debug section of a Mach-O object whose value counts from the
/// start of the section of the code or data it points at; the address of
/// that section in the object, known once the object is written, goes into
/// it then.
#[derive(Debug, Clone)]
pub struct SectionPlace
{
    /// The name of the debug section, such as `__debug_line`.
    section: String,
    /// The offset of the place in the section.
    offset: u64,
    /// The size of the place, in octets.
    size: u8,
    /// The name of the section it counts from, such as `__text`.
    base: String,
}

/// What a debug section points at: a function or a global, or another debug
/// section.
#[derive(Debug, Clone, Copy)]
enum Target
{
    /// The function at this index of the line table, or, past the functions,
    /// the global at this index less their count.
    Code(usize),
    /// The start of a debug section.
    Section(SectionId),
}

/// One place of a debug section that the linker fills in.
#[derive(Debug, Clone, Copy)]
struct Reloc
{
    /// The offset of the place in its section.
    offset: u64,
    /// The size of the place, in octets.
    size: u8,
    /// What it points at.
    target: Target,
    /// The offset from the start of the target.
    addend: i64,
}

/// A writer of a debug section that keeps the places the linker fills in.
/// Mach-O reads the offset of one debug section in another as it stands, so
/// the writer writes those offsets with no place to fill.
#[derive(Debug, Clone)]
struct RelocWriter
{
    /// The octets.
    out: EndianVec<RunTimeEndian>,
    /// The places to fill in.
    relocs: Vec<Reloc>,
    /// Whether the object is Mach-O.
    macho: bool,
}

impl Writer for RelocWriter
{
    type Endian = RunTimeEndian;

    fn endian(&self) -> Self::Endian { self.out.endian() }

    fn len(&self) -> usize { self.out.len() }

    fn write(&mut self, octets: &[u8]) -> gimli::write::Result<()>
    {
        self.out.write(octets)
    }

    fn write_at(&mut self, offset: usize, octets: &[u8]) -> gimli::write::Result<()>
    {
        self.out.write_at(offset, octets)
    }

    fn write_address(&mut self, address: Address, size: u8)
    -> gimli::write::Result<()>
    {
        let Address::Symbol { symbol, addend } = address
        else
        {
            let Address::Constant(value) = address
            else
            {
                return Ok(());
            };
            return self.write_udata(value, size);
        };
        let offset = self.len() as u64;
        self.relocs.push(Reloc { offset, size, target: Target::Code(symbol), addend });
        self.write_udata(0, size)
    }

    fn write_offset(
        &mut self, val: usize, section: SectionId, size: u8,
    ) -> gimli::write::Result<()>
    {
        let offset = self.len();
        self.write_udata(0, size)?;
        self.write_offset_at(offset, val, section, size)
    }

    fn write_offset_at(
        &mut self, offset: usize, val: usize, section: SectionId, size: u8,
    ) -> gimli::write::Result<()>
    {
        ret_if!(self.macho, self.write_udata_at(offset, val as u64, size));
        let target = Target::Section(section);
        let reloc = Reloc { offset: offset as u64, size, target, addend: val as i64 };
        self.relocs.push(reloc);
        self.write_udata_at(offset, 0, size)
    }
}

/// Add the DWARF of the line table to an object.
///
/// # Arguments
///
/// * `product` - The object, once each function is in it.
/// * `table` - The line table.
///
/// # Returns
///
/// The places [`place_sections`] fills in once the object is written, or the
/// error of the DWARF or of the object.
pub fn write(
    product: &mut ObjectProduct, table: &LineTable,
) -> Result<Vec<SectionPlace>, Error>
{
    ret_if!(table.funcs.is_empty() || table.files.is_empty(), Ok(Vec::new()));
    let fns = table.funcs.iter().map(|f| product.function_symbol(f.id));
    let data = table.globals.iter().map(|g| product.data_symbol(g.id));
    let syms: Vec<object::write::SymbolId> = fns.chain(data).collect();
    let obj = &mut product.object;
    // Each CPU Cranelift builds for here is little-endian.
    let endian = RunTimeEndian::Little;
    let address_size = obj.architecture().address_size().map_or(8, |s| s.bytes());
    let encoding = Encoding { format: Format::Dwarf32, version: 4, address_size };
    let frame = frame_register(obj.architecture());
    let mut dwarf = unit(encoding, table, frame);
    let writer = RelocWriter {
        out: EndianVec::new(endian),
        relocs: Vec::new(),
        macho: obj.format() == BinaryFormat::MachO,
    };
    let mut sections = Sections::new(writer);
    dwarf.write(&mut sections).map_err(module_error)?;
    let mut ids = HashMap::new();
    sections
        .for_each(|id, w: &RelocWriter| add_section(obj, &mut ids, id, w))
        .map_err(module_error)?;
    let mut places = Vec::new();
    sections
        .for_each(|id, w: &RelocWriter| {
            add_relocs(obj, &ids, &syms, id, w, &mut places)
        })
        .map_err(module_error)?;
    Ok(places)
}

/// Add the address of a section of a written Mach-O object, such as `__text`,
/// to each place of its debug info that counts from the start of that
/// section, as a debugger reads the debug info of an object as it stands.
///
/// # Arguments
///
/// * `octets` - The object.
/// * `places` - The places.
///
/// # Returns
///
/// `Ok`, or the error of an object that does not read back.
pub fn place_sections(octets: &mut [u8], places: &[SectionPlace]) -> Result<(), Error>
{
    use object::Object;
    use object::ObjectSection;

    ret_if!(places.is_empty(), Ok(()));
    let file = object::File::parse(&*octets).map_err(module_error)?;
    let start = |p: &SectionPlace| {
        let base = file.section_by_name(&p.base).map_or(0, |s| s.address());
        let section = file.section_by_name(&p.section)?;
        section.file_range().map(|(at, _)| (at + p.offset, p.size, base))
    };
    let at: Vec<(u64, u8, u64)> = places.iter().filter_map(start).collect();
    drop(file);
    do_for!((offset, size, base) in at, add_at(octets, offset as usize, size as usize, base));
    Ok(())
}

/// Add a value to the little-endian number at a place of an object.
///
/// # Arguments
///
/// * `octets` - The object.
/// * `at` - The offset of the place.
/// * `size` - The size of the place, 4 or 8.
/// * `value` - The value to add.
fn add_at(octets: &mut [u8], at: usize, size: usize, value: u64)
{
    let Some(place) = octets.get_mut(at..at + size)
    else
    {
        return;
    };
    let mut word = [0u8; 8];
    word[..size].copy_from_slice(place);
    let sum = u64::from_le_bytes(word).wrapping_add(value).to_le_bytes();
    place.copy_from_slice(&sum[..size]);
}

/// The compile unit of a program: a subprogram for each function, and the
/// line program of its code.
///
/// # Arguments
///
/// * `encoding` - The form of the DWARF.
/// * `table` - The line table.
/// * `frame` - The frame pointer of the CPU, or `None` for a CPU whose locals
///   get no place.
///
/// # Returns
///
/// The unit.
fn unit(
    encoding: Encoding, table: &LineTable, frame: Option<gimli::Register>,
) -> DwarfUnit
{
    let first = table.files[0].clone();
    let dir = std::path::Path::new(&first).parent().map(|p| p.display().to_string());
    let dir = dir.unwrap_or_default().into_bytes();
    let mut dwarf = DwarfUnit::new(encoding);
    let mut lines = LineProgram::new(
        encoding,
        LineEncoding::default(),
        LineString::String(dir.clone()),
        None,
        LineString::String(first.clone().into_bytes()),
        None,
    );
    let at = lines.default_directory();
    let file = |f: &String| {
        lines.add_file(LineString::String(f.clone().into_bytes()), at, None)
    };
    let files: Vec<_> = table.files.iter().map(file).collect();
    let mut kinds = Kinds::new(table.platform);
    let mut ranges = Vec::new();
    for (n, f) in table.funcs.iter().enumerate()
    {
        let begin = Address::Symbol { symbol: n, addend: 0 };
        lines.begin_sequence(Some(begin));
        let site = |s: u32| table.sites.get(s as usize).copied();
        let rows = f.rows.iter().filter_map(|(at, s)| site(*s).map(|s| (*at, s)));
        for (offset, (file, line, col)) in rows.filter(|(_, (_, line, _))| *line > 0)
        {
            let row = lines.row();
            row.address_offset = u64::from(offset);
            row.file = files[file as usize];
            row.line = u64::from(line);
            row.column = u64::from(col);
            lines.generate_row();
        }
        lines.end_sequence(u64::from(f.size));
        ranges.push(Range::StartLength { begin, length: u64::from(f.size) });
        let routine = dwarf.unit.add(dwarf.unit.root(), gimli::DW_TAG_subprogram);
        let die = dwarf.unit.get_mut(routine);
        die.set(gimli::DW_AT_name, AttributeValue::String(f.name.clone().into_bytes()));
        die.set(gimli::DW_AT_low_pc, AttributeValue::Address(begin));
        die.set(gimli::DW_AT_high_pc, AttributeValue::Udata(u64::from(f.size)));
        let Some(frame) = frame.filter(|_| !f.vars.is_empty())
        else
        {
            continue;
        };
        let mut base = Expression::new();
        base.op_reg(frame);
        die.set(gimli::DW_AT_frame_base, AttributeValue::Exprloc(base));
        let decl = |v: &FnVar| {
            let (file, line, _) = site(v.site?)?;
            Some((files[file as usize], u64::from(line)))
        };
        do_for!(v in &f.vars, add_var(&mut dwarf.unit, &mut kinds, routine, v, decl(v)));
    }
    let globals = table.globals.iter().enumerate();
    do_for!((n, g) in globals.filter(|(_, g)| !g.name.starts_with('%')), {
        let decl = g.site.and_then(|s| table.sites.get(s as usize).copied()).map(|(file, line, _)| (files[file as usize], u64::from(line)));
        let symbol = table.funcs.len() + n;
        add_global(&mut dwarf.unit, &mut kinds, g, Address::Symbol { symbol, addend: 0 }, decl);
    });
    dwarf.unit.line_program = lines;
    let ranges = dwarf.unit.ranges.add(RangeList(ranges));
    let root = dwarf.unit.root();
    let die = dwarf.unit.get_mut(root);
    die.set(gimli::DW_AT_producer, AttributeValue::String(b"zakac".to_vec()));
    // The linker of macOS keeps no debug map of a unit with no language, and C
    // is the nearest a debugger reads.
    die.set(gimli::DW_AT_language, AttributeValue::Language(gimli::DW_LANG_C99));
    die.set(gimli::DW_AT_name, AttributeValue::String(first.into_bytes()));
    die.set(gimli::DW_AT_comp_dir, AttributeValue::String(dir));
    die.set(gimli::DW_AT_low_pc, AttributeValue::Address(Address::Constant(0)));
    die.set(gimli::DW_AT_ranges, AttributeValue::RangeListRef(ranges));
    dwarf
}

/// The DWARF number of the frame pointer of a CPU, which the locals of a
/// function count from: each build with debug info keeps one.
///
/// # Arguments
///
/// * `arch` - The CPU.
///
/// # Returns
///
/// The register, or `None` for a CPU Cranelift builds for no other.
fn frame_register(arch: Architecture) -> Option<gimli::Register>
{
    match arch
    {
        Architecture::Aarch64 => Some(gimli::AArch64::X29),
        Architecture::X86_64 => Some(gimli::X86_64::RBP),
        Architecture::Riscv64 => Some(gimli::RiscV::S0),
        _ => None,
    }
}

/// Add a parameter or a local to the DIE of its function: its name, its
/// kind, its declaration and its place from the frame pointer.
///
/// # Arguments
///
/// * `unit` - The unit.
/// * `kinds` - The DIEs of the kinds so far.
/// * `routine` - The DIE of the function.
/// * `var` - The parameter or local.
/// * `decl` - The file and line of its declaration.
fn add_var(
    unit: &mut gimli::write::Unit, kinds: &mut Kinds,
    routine: gimli::write::UnitEntryId, var: &FnVar,
    decl: Option<(gimli::write::FileId, u64)>,
)
{
    let kind = kinds.die_of(unit, var.kind);
    let tag = tern!(var.param, gimli::DW_TAG_formal_parameter, gimli::DW_TAG_variable);
    let id = unit.add(routine, tag);
    let die = unit.get_mut(id);
    die.set(gimli::DW_AT_name, AttributeValue::String(var.name.clone().into_bytes()));
    die.set(gimli::DW_AT_type, AttributeValue::UnitRef(kind));
    let mut at = Expression::new();
    at.op_fbreg(var.at);
    die.set(gimli::DW_AT_location, AttributeValue::Exprloc(at));
    let Some((file, line)) = decl
    else
    {
        return;
    };
    die.set(gimli::DW_AT_decl_file, AttributeValue::FileIndex(Some(file)));
    die.set(gimli::DW_AT_decl_line, AttributeValue::Udata(line));
}

/// Add a global to the unit: its name, its kind, its declaration and its
/// address.
///
/// # Arguments
///
/// * `unit` - The unit.
/// * `kinds` - The DIEs of the kinds so far.
/// * `global` - The global.
/// * `addr` - Its address.
/// * `decl` - The file and line of its declaration.
fn add_global(
    unit: &mut gimli::write::Unit, kinds: &mut Kinds, global: &GlobalVar,
    addr: Address, decl: Option<(gimli::write::FileId, u64)>,
)
{
    let kind = kinds.die_of(unit, global.kind);
    let root = unit.root();
    let id = unit.add(root, gimli::DW_TAG_variable);
    let die = unit.get_mut(id);
    die.set(
        gimli::DW_AT_name,
        AttributeValue::String(global.name.clone().into_bytes()),
    );
    die.set(gimli::DW_AT_type, AttributeValue::UnitRef(kind));
    let mut at = Expression::new();
    at.op_addr(addr);
    die.set(gimli::DW_AT_location, AttributeValue::Exprloc(at));
    let Some((file, line)) = decl
    else
    {
        return;
    };
    die.set(gimli::DW_AT_decl_file, AttributeValue::FileIndex(Some(file)));
    die.set(gimli::DW_AT_decl_line, AttributeValue::Udata(line));
}

/// Add one debug section to the object, unless it is blank.
///
/// # Arguments
///
/// * `obj` - The object.
/// * `ids` - The section of the object of each debug section so far.
/// * `id` - The debug section.
/// * `w` - Its octets.
///
/// # Returns
///
/// `Ok`.
fn add_section(
    obj: &mut object::write::Object<'static>,
    ids: &mut HashMap<SectionId, object::write::SectionId>, id: SectionId,
    w: &RelocWriter,
) -> Result<(), String>
{
    ret_if!(w.out.slice().is_empty(), Ok(()));
    let segment = obj.segment_name(StandardSegment::Debug).to_vec();
    // Mach-O spells `.debug_line` as `__debug_line`.
    let macho = obj.format() == BinaryFormat::MachO;
    let name = tern!(macho, id.name().replace('.', "__"), id.name().to_string());
    let section = obj.add_section(segment, name.into_bytes(), SectionKind::Debug);
    obj.append_section_data(section, w.out.slice(), 1);
    ids.insert(id, section);
    Ok(())
}

/// Add the places the linker fills in of one debug section.
///
/// # Arguments
///
/// * `obj` - The object.
/// * `ids` - The section of the object of each debug section.
/// * `syms` - The name of each function in the object, in the order of the line
///   table, then of each global.
/// * `id` - The debug section.
/// * `w` - Its octets and places.
///
/// # Returns
///
/// `Ok`, or the error of the object.
fn add_relocs(
    obj: &mut object::write::Object<'static>,
    ids: &HashMap<SectionId, object::write::SectionId>,
    syms: &[object::write::SymbolId], id: SectionId, w: &RelocWriter,
    places: &mut Vec<SectionPlace>,
) -> Result<(), String>
{
    let Some(&section) = ids.get(&id)
    else
    {
        return Ok(());
    };
    let coff = obj.format() == BinaryFormat::Coff;
    let macho = obj.format() == BinaryFormat::MachO;
    do_for!(r in &w.relocs, {
        let (symbol, kind, addend) = match r.target
        {
            Target::Code(n) if macho =>
            {
                let (to, kind, addend, base) = in_section(obj, syms[n], r.addend);
                let section = id.name().replace('.', "__");
                places.push(SectionPlace { section, offset: r.offset, size: r.size, base });
                (to, kind, addend)
            }
            Target::Code(n) => (syms[n], RelocationKind::Absolute, r.addend),
            Target::Section(s) =>
            {
                let Some(&to) = ids.get(&s) else { continue };
                // COFF names the place inside its section; ELF adds the start.
                let kind = tern!(coff, RelocationKind::SectionOffset, RelocationKind::Absolute);
                (obj.section_symbol(to), kind, r.addend)
            }
        };
        let flags = RelocationFlags::Generic {
            kind,
            encoding: RelocationEncoding::Generic,
            size: r.size * 8,
        };
        let reloc = Relocation { offset: r.offset, symbol, addend, flags };
        obj.add_relocation(section, reloc).map_err(|e| e.to_string())?;
    });
    Ok(())
}

/// The place of a function or a global in a Mach-O object, as clang writes it
/// in debug info: against its section, with its offset in the section as the
/// value in place. A debugger reads the debug info of the object as it
/// stands, with no relocation applied, and finds each function and global at
/// that offset through the debug map of the executable.
///
/// # Arguments
///
/// * `obj` - The object.
/// * `at` - The name of the function or global in the object.
/// * `addend` - The offset from its start.
///
/// # Returns
///
/// What to relocate against, the kind of relocation, the addend, and the
/// name of the section, whose address the place counts from.
fn in_section(
    obj: &mut object::write::Object<'static>, at: object::write::SymbolId, addend: i64,
) -> (object::write::SymbolId, RelocationKind, i64, String)
{
    let found = obj.symbol(at);
    let object::write::SymbolSection::Section(section) = found.section
    else
    {
        return (at, RelocationKind::Absolute, addend, String::new());
    };
    let value = found.value as i64 + addend;
    let base = obj.section(section).name().unwrap_or_default().to_string();
    (obj.section_symbol(section), RelocationKind::Absolute, value, base)
}
