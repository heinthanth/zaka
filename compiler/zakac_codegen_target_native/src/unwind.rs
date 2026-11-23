//! The unwind tables of 64-bit Windows for the code of an object: `.xdata`, the
//! unwind codes of each function, and `.pdata`, a row of each function. On x64
//! a row holds the start, the end and the codes of its function; on ARM64 it
//! holds the start and the codes, and the codes start with a header word. A
//! debugger walks the frames of the code through them, and a `longjmp` or an
//! exception unwinds through them; cranelift-object writes none on Windows.

use cranelift_codegen::CompiledCode;
use cranelift_codegen::isa::TargetIsa;
use cranelift_codegen::isa::unwind::UnwindInfo;
use cranelift_module::FuncId;
use cranelift_object::ObjectProduct;
use object::Architecture;
use object::BinaryFormat;
use object::RelocationEncoding;
use object::RelocationFlags;
use object::RelocationKind;
use object::SectionKind;
use object::write::Relocation;
use object::write::SectionId;
use object::write::SymbolId;
use zakac_host_errors::Error;
use zakac_rust_sugars::ret_if;

use crate::emit::module_error;

/// The size of a row of `.pdata` on x64: three image offsets of four octets.
const ROW: usize = 12;
/// The size of a row of `.pdata` on ARM64: two image offsets of four octets.
const ARM_ROW: usize = 8;
/// The unwind code `end` of ARM64, which also fills the last word of codes.
const ARM_END: u8 = 0xE4;

/// The unwind codes of one function of 64-bit Windows.
#[derive(Debug, Clone)]
pub struct FnUnwind
{
    /// The function.
    pub id: FuncId,
    /// The size of its code.
    pub size: u32,
    /// Its `UNWIND_INFO`, as `.xdata` holds it.
    pub xdata: Vec<u8>,
}

/// The unwind codes of a compiled function, for a target of 64-bit Windows.
///
/// # Arguments
///
/// * `id` - The function.
/// * `code` - Its code.
/// * `isa` - The target.
///
/// # Returns
///
/// The codes, or `None` for another target.
pub fn fn_unwind(
    id: FuncId, code: &CompiledCode, isa: &dyn TargetIsa,
) -> Option<FnUnwind>
{
    let size = code.code_info().total_size;
    let xdata = match code.create_unwind_info(isa)
    {
        Ok(Some(UnwindInfo::WindowsX64(info))) =>
        {
            let mut xdata = vec![0; info.emit_size()];
            info.emit(&mut xdata);
            xdata
        }
        Ok(Some(UnwindInfo::WindowsArm64(info))) =>
        {
            arm_xdata(size, info.code_words(), |buf| info.emit(buf))?
        }
        _ => return None,
    };
    Some(FnUnwind { id, size, xdata })
}

/// The `.xdata` of a function of ARM64: a header word, then the unwind codes,
/// with `end` after them up to the end of their last word. The header holds
/// the length of the function in words of four octets and the count of the
/// words of codes; it describes no epilog, as a frame is unwound from a call
/// inside its code.
///
/// # Arguments
///
/// * `size` - The size of the code of the function.
/// * `words` - The count of words the codes take.
/// * `emit` - What writes the codes.
///
/// # Returns
///
/// The octets, or `None` for a function too long, or with more codes than
/// one header word counts.
fn arm_xdata(size: u32, words: u8, emit: impl FnOnce(&mut [u8])) -> Option<Vec<u8>>
{
    // One more word than the codes take keeps room for an `end`.
    let words = u32::from(words) + 1;
    let length = size / 4;
    ret_if!(length >= 1 << 18 || words >= 1 << 5, None);
    let mut codes = vec![ARM_END; words as usize * 4];
    emit(&mut codes);
    let header = length | (words << 27);
    Some(header.to_le_bytes().into_iter().chain(codes).collect())
}

/// Add the unwind tables of 64-bit Windows to a COFF object.
///
/// # Arguments
///
/// * `product` - The object, once each function is in it.
/// * `tables` - The unwind codes of each function.
///
/// # Returns
///
/// `Ok`, or the error of a relocation.
pub fn write(product: &mut ObjectProduct, tables: &[FnUnwind]) -> Result<(), Error>
{
    ret_if!(tables.is_empty() || product.object.format() != BinaryFormat::Coff, Ok(()));
    let starts: Vec<SymbolId> =
        tables.iter().map(|t| product.function_symbol(t.id)).collect();
    let obj = &mut product.object;
    let xdata =
        obj.add_section(Vec::new(), b".xdata".to_vec(), SectionKind::ReadOnlyData);
    let pdata =
        obj.add_section(Vec::new(), b".pdata".to_vec(), SectionKind::ReadOnlyData);
    let codes = obj.section_symbol(xdata);
    let arm = obj.architecture() == Architecture::Aarch64;
    for (t, start) in tables.iter().zip(starts)
    {
        let at = obj.append_section_data(xdata, &t.xdata, 4);
        let row = obj.append_section_data(
            pdata,
            &[0; ROW][..if arm { ARM_ROW } else { ROW }],
            4,
        );
        image_offset(obj, pdata, row, start, 0)?;
        if arm
        {
            image_offset(obj, pdata, row + 4, codes, at as i64)?;
            continue;
        }
        image_offset(obj, pdata, row + 4, start, i64::from(t.size))?;
        image_offset(obj, pdata, row + 8, codes, at as i64)?;
    }
    Ok(())
}

/// Add an image offset of four octets (`IMAGE_REL_AMD64_ADDR32NB`, or
/// `IMAGE_REL_ARM64_ADDR32NB` on ARM64) to a section.
///
/// # Arguments
///
/// * `obj` - The object.
/// * `section` - The section of the place.
/// * `offset` - The place.
/// * `target` - What the offset counts to.
/// * `addend` - The octets past the target.
///
/// # Returns
///
/// `Ok`, or the error of the object.
fn image_offset(
    obj: &mut object::write::Object<'static>, section: SectionId, offset: u64,
    target: SymbolId, addend: i64,
) -> Result<(), Error>
{
    let flags = RelocationFlags::Generic {
        kind: RelocationKind::ImageOffset,
        encoding: RelocationEncoding::Generic,
        size: 32,
    };
    let reloc = Relocation { offset, symbol: target, addend, flags };
    obj.add_relocation(section, reloc).map_err(module_error)
}
