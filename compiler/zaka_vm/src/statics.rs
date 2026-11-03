use zakac_codegen::bc::StaticCell;
use zakac_codegen::bc::StaticSeq;
use zakac_ir_builder::value::Value;
use zakac_ir_builder::value::to_raw;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;
use zakac_semcheck::kind::Width;

/// Where the static blocks placed so far start, and what each holds.
#[derive(Debug, Clone, Copy, bon::Builder)]
pub(crate) struct Placed<'a>
{
    /// The start of the block of each static text.
    pub texts: &'a [u64],
    /// The u8 count of each static text.
    pub text_lens: &'a [u64],
    /// The start of the block of each static list placed so far.
    pub seqs: &'a [u64],
    /// The cell count of each static list.
    pub seq_lens: &'a [u64],
    /// Bit count of the platform-sized kinds.
    pub platform: u16,
}

/// The octets of the frozen block of a static list: its cells one after
/// another, or the fields of a struct at their offsets, text and plain list
/// cells as three words that point at their own static blocks with a room
/// of 0.
///
/// # Arguments
///
/// * `seq` - The static list.
/// * `placed` - The static blocks placed before it.
///
/// # Returns
///
/// The octets, or `None` when a cell names a block not placed so far.
pub(crate) fn seq_octets(seq: &StaticSeq, placed: Placed) -> Option<Vec<u8>>
{
    let Some(cell) = seq.kind.cell_kind()
    else
    {
        // A struct literal: its fields at their offsets.
        let mut out = vec![0; seq.kind.size(placed.platform) as usize];
        return put_parts(&seq.cells, seq.kind, &mut out, 0, placed).map(|()| out);
    };
    let stride = cell.size(placed.platform) as usize;
    let mut out = vec![0; stride * seq.cells.len()];
    for (i, c) in seq.cells.iter().enumerate()
    {
        put(c, cell, &mut out, i * stride, placed)?;
    }
    Some(out)
}

/// Write one cell, or one part of a cell, at an offset.
///
/// # Arguments
///
/// * `cell` - The cell.
/// * `kind` - Its kind.
/// * `out` - The octets of the block.
/// * `at` - The offset of the cell.
/// * `placed` - The static blocks placed so far.
///
/// # Returns
///
/// `Some`, or `None` when the cell names a block not placed so far.
fn put(
    cell: &StaticCell, kind: DataKind, out: &mut [u8], at: usize, placed: Placed,
) -> Option<()>
{
    match cell
    {
        StaticCell::Value(v) => write(out, at, &to_raw(*v, placed.platform)),
        StaticCell::Text(t) =>
        {
            let t = *t as usize;
            let (start, len) = (*placed.texts.get(t)?, *placed.text_lens.get(t)?);
            words(out, at, start, len, placed.platform)
        }
        StaticCell::Seq(s) =>
        {
            let s = *s as usize;
            let (start, count) = (*placed.seqs.get(s)?, *placed.seq_lens.get(s)?);
            words(out, at, start, count, placed.platform)
        }
        StaticCell::Parts(parts) => put_parts(parts, kind, out, at, placed),
        StaticCell::Blank => Some(()),
    }
}

/// Write the cells of a counted list or the fields of a struct.
///
/// # Arguments
///
/// * `parts` - The cells or the fields, in order.
/// * `kind` - The counted list or struct kind.
/// * `out` - The octets of the block.
/// * `at` - The offset of the value.
/// * `placed` - The static blocks placed so far.
///
/// # Returns
///
/// `Some`, or `None` when a part names a block not placed so far.
fn put_parts(
    parts: &[StaticCell], kind: DataKind, out: &mut [u8], at: usize, placed: Placed,
) -> Option<()>
{
    let platform = placed.platform;
    if let Some(list) = kind.as_list()
    {
        let stride = list.cell.size(platform) as usize;
        let mut each = parts.iter().enumerate();
        return each
            .try_for_each(|(i, p)| put(p, list.cell, out, at + i * stride, placed));
    }
    let fields = kind.as_struct()?;
    let mut each = parts.iter().zip(fields.fields()).enumerate();
    each.try_for_each(|(i, (p, (_, k)))| {
        put(p, *k, out, at + fields.offset(i, platform) as usize, placed)
    })
}

/// Write the three words of text or a plain list: the start of its cells,
/// their count, and a room of 0.
///
/// # Arguments
///
/// * `out` - The octets of the block.
/// * `at` - The offset of the first word.
/// * `start` - The start of the cells.
/// * `count` - The count of the cells.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// `Some`, or `None` when the words do not fit.
fn words(out: &mut [u8], at: usize, start: u64, count: u64, platform: u16)
-> Option<()>
{
    let word = usize::from(platform / 8);
    let usize = IntKind::new(false, Width::Platform);
    write(out, at, &to_raw(Value::Ptr(start), platform))?;
    write(
        out,
        at + word,
        &to_raw(Value::int(usize, u128::from(count), platform), platform),
    )
}

/// Write octets into the block at an offset.
///
/// # Arguments
///
/// * `out` - The octets of the block.
/// * `at` - The offset.
/// * `raw` - The octets.
///
/// # Returns
///
/// `Some`, or `None` when the octets do not fit.
fn write(out: &mut [u8], at: usize, raw: &[u8]) -> Option<()>
{
    out.get_mut(at..at + raw.len())?.copy_from_slice(raw);
    Some(())
}
