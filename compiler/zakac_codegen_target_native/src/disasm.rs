//! The listing of the machine code of an object: each function under its
//! name, with its instructions, and the target of each place the linker fills
//! in, since the code of an object calls address 0 until it is linked. x86-64
//! reads through iced-x86 and aarch64 through yaxpeax-arm; another CPU has no
//! listing.

use std::collections::BTreeMap;
use std::fmt::Write;

use iced_x86::Decoder as X86Decoder;
use iced_x86::DecoderOptions;
use iced_x86::Formatter;
use iced_x86::Instruction;
use iced_x86::IntelFormatter;
use object::Architecture;
use object::BinaryFormat;
use object::Object;
use object::ObjectSection;
use object::ObjectSymbol;
use object::RelocationTarget;
use object::SectionKind;
use object::SymbolKind;
use yaxpeax_arch::Decoder as _;
use yaxpeax_arch::U8Reader;
use yaxpeax_arm::armv8::a64::InstDecoder;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

/// The width of an instruction of aarch64.
const A64_WIDTH: usize = 4;

/// The code of one object, as the listing reads it.
struct Code<'a>
{
    /// The octets of its code section.
    octets: &'a [u8],
    /// The CPU.
    arch: Architecture,
    /// The name of each place the linker fills in, at its offset.
    targets: BTreeMap<u64, String>,
}

/// The listing of the code of an object.
///
/// # Arguments
///
/// * `octets` - The object, ELF, Mach-O or COFF.
/// * `name_of` - The name to show for a name of the object, such as the Zaka
///   name of `zk_f0`.
///
/// # Returns
///
/// The listing, or `None` for an object that does not read or a CPU with no
/// listing.
pub fn listing(octets: &[u8], name_of: &dyn Fn(&str) -> String) -> Option<String>
{
    let file = object::File::parse(octets).ok()?;
    let arch = file.architecture();
    ret_if!(!matches!(arch, Architecture::X86_64 | Architecture::Aarch64), None);
    let text = file.sections().find(|s| s.kind() == SectionKind::Text)?;
    let base = text.address();
    // Mach-O puts `_` before each C name.
    let macho = file.format() == BinaryFormat::MachO;
    let shown =
        |raw: &str| name_of(tern!(macho, raw.strip_prefix('_').unwrap_or(raw), raw));
    let in_text = |s: &object::Symbol<'_, '_>| {
        s.section_index() == Some(text.index()) && s.kind() == SymbolKind::Text
    };
    let functions = file.symbols().filter(in_text);
    let starts: BTreeMap<u64, String> = functions
        .filter_map(|s| Some((s.address() - base, shown(s.name().ok()?))))
        .collect();
    let target = |(at, r): (u64, object::Relocation)| {
        let RelocationTarget::Symbol(index) = r.target()
        else
        {
            return None;
        };
        let name = file.symbol_by_index(index).ok()?.name().ok()?.to_string();
        Some((at, shown(&name)))
    };
    let targets = text.relocations().filter_map(target).collect();
    let code = Code { octets: text.data().ok()?, arch, targets };
    let ends = starts.keys().skip(1).copied().chain([code.octets.len() as u64]);
    let each = starts.iter().zip(ends);
    Some(each.map(|((start, name), end)| code.function(name, *start, end)).collect())
}

impl Code<'_>
{
    /// The listing of one function: its name, then a line for each
    /// instruction.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    /// * `start` - Its offset in the code section.
    /// * `end` - The offset after it.
    ///
    /// # Returns
    ///
    /// The text.
    fn function(&self, name: &str, start: u64, end: u64) -> String
    {
        let octets = &self.octets[start as usize..end as usize];
        let lines = tern!(
            self.arch == Architecture::X86_64,
            self.x86(octets, start),
            self.a64(octets, start)
        );
        format!("{name}:\n{lines}\n")
    }

    /// The lines of x86-64 code, in Intel form.
    ///
    /// # Arguments
    ///
    /// * `octets` - The code.
    /// * `start` - Its offset in the code section.
    ///
    /// # Returns
    ///
    /// The lines.
    fn x86(&self, octets: &[u8], start: u64) -> String
    {
        let mut decoder = X86Decoder::with_ip(64, octets, start, DecoderOptions::NONE);
        let mut formatter = IntelFormatter::new();
        let mut instruction = Instruction::default();
        let mut out = String::new();
        while decoder.can_decode()
        {
            decoder.decode_out(&mut instruction);
            let mut text = String::new();
            formatter.format(&instruction, &mut text);
            let (at, len) = (instruction.ip(), instruction.len() as u64);
            out.push_str(&self.line(at, len, &text));
        }
        out
    }

    /// The lines of aarch64 code.
    ///
    /// # Arguments
    ///
    /// * `octets` - The code.
    /// * `start` - Its offset in the code section.
    ///
    /// # Returns
    ///
    /// The lines.
    fn a64(&self, octets: &[u8], start: u64) -> String
    {
        let decoder = InstDecoder::default();
        let line = |(i, word): (usize, &[u8])| {
            let at = start + (i * A64_WIDTH) as u64;
            let decoded = decoder.decode(&mut U8Reader::new(word));
            let text = decoded
                .map_or_else(|_| format!(".word {word:02x?}"), |d| d.to_string());
            self.line(at, A64_WIDTH as u64, &text)
        };
        octets.chunks(A64_WIDTH).enumerate().map(line).collect()
    }

    /// One line of the listing: the offset, the instruction, and the target
    /// of a place inside it that the linker fills in.
    ///
    /// # Arguments
    ///
    /// * `at` - The offset of the instruction.
    /// * `len` - Its length.
    /// * `text` - The instruction.
    ///
    /// # Returns
    ///
    /// The line.
    fn line(&self, at: u64, len: u64, text: &str) -> String
    {
        let mut line = format!("  {at:6x}  {text}");
        let target = self.targets.range(at..at + len).next();
        do_if!(let Some((_, name)) = target, _ = write!(line, "  ; {name}"));
        line.push('\n');
        line
    }
}
