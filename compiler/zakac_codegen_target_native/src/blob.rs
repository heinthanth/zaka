//! Data the generated code reads, set out as C sets out its structs: octets
//! with the addresses of other data and of functions at their offsets.

use cranelift_module::DataDescription;
use cranelift_module::DataId;
use cranelift_module::FuncId;
use cranelift_module::Module;
use cranelift_module::ModuleResult;

/// What an address in data names.
#[derive(Clone, Copy)]
pub enum Target
{
    /// Data, at an offset.
    Data(DataId, i64),
    /// A function.
    Func(FuncId),
}

/// The octets of a piece of data, with the addresses in it.
pub struct Blob
{
    /// The octets.
    pub octets: Vec<u8>,
    /// Each address: its offset in the octets, and what it names.
    pub relocs: Vec<(u32, Target)>,
    /// The alignment.
    pub align: u64,
}

impl Blob
{
    /// A blob with no octets, aligned to a word.
    ///
    /// # Returns
    ///
    /// The blob.
    pub fn new() -> Self { Blob { octets: Vec::new(), relocs: Vec::new(), align: 8 } }

    /// Pad the octets to a multiple of `n`.
    ///
    /// # Arguments
    ///
    /// * `n` - The alignment.
    fn align_to(&mut self, n: usize)
    {
        let len = self.octets.len().div_ceil(n) * n;
        self.octets.resize(len, 0);
    }

    /// Add a 32-bit number.
    ///
    /// # Arguments
    ///
    /// * `v` - The number.
    pub fn u32(&mut self, v: u32)
    {
        self.align_to(4);
        self.octets.extend(v.to_le_bytes());
    }

    /// Add a 64-bit number or a word.
    ///
    /// # Arguments
    ///
    /// * `v` - The number.
    pub fn u64(&mut self, v: u64)
    {
        self.align_to(8);
        self.octets.extend(v.to_le_bytes());
    }

    /// Add an address, or NULL.
    ///
    /// # Arguments
    ///
    /// * `t` - What it names.
    pub fn ptr(&mut self, t: Option<Target>)
    {
        self.align_to(8);
        if let Some(t) = t
        {
            self.relocs.push((self.octets.len() as u32, t));
        }
        self.octets.extend([0u8; 8]);
    }

    /// Pad the blob to its alignment, as C pads the end of a struct.
    ///
    /// # Returns
    ///
    /// The blob.
    pub fn done(mut self) -> Self
    {
        self.align_to(8);
        self
    }
}

/// Define data from a blob.
///
/// # Arguments
///
/// * `m` - The module.
/// * `id` - The data, declared.
/// * `blob` - Its octets and addresses.
///
/// # Returns
///
/// `Ok`, or the error of the module.
pub fn define<M: Module>(m: &mut M, id: DataId, blob: Blob) -> ModuleResult<()>
{
    let mut d = DataDescription::new();
    let size = blob.octets.len().max(1);
    let mut octets = blob.octets;
    octets.resize(size, 0);
    d.define(octets.into_boxed_slice());
    d.set_align(blob.align);
    for (offset, target) in blob.relocs
    {
        match target
        {
            Target::Data(data, addend) =>
            {
                let gv = m.declare_data_in_data(data, &mut d);
                d.write_data_addr(offset, gv, addend);
            }
            Target::Func(func) =>
            {
                let f = m.declare_func_in_data(func, &mut d);
                d.write_function_addr(offset, f);
            }
        }
    }
    m.define_data(id, &d)
}
