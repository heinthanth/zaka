use std::fmt;

use bon::Builder;
use zakac_i18n::t;

/// The reason a program stopped before its end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapKind
{
    /// An integer division or remainder with zero as divisor.
    ZeroDivisor,
    /// A shift distance not below the bit width.
    ShiftTooFar
    {
        /// The distance.
        distance: u128,
        /// The bit width of the shifted value.
        bits: u16,
    },
    /// The calls nest too deep.
    StackOverflow
    {
        /// The limit.
        depth: u32,
    },
    /// The op budget ran out.
    StepLimit
    {
        /// The limit.
        steps: u64,
    },
    /// A counted loop step is zero or negative.
    StepNotPositive,
    /// The ops break a rule the generator keeps, such as a slot index out
    /// of range.
    Malformed
    {
        /// What is wrong.
        reason: String,
    },
    /// The output could not be written.
    Write
    {
        /// The I/O error text.
        msg: String,
    },
    /// An access below the first block, in the reserved region.
    NullDeref
    {
        /// The address.
        addr: u64,
    },
    /// An access outside each block, or across the end of one.
    OutOfBounds
    {
        /// The address.
        addr: u64,
        /// The octet count of the access.
        size: u32,
    },
    /// An access inside a freed block.
    UseAfterFree
    {
        /// The address.
        addr: u64,
    },
    /// A free of a heap block that was freed before.
    DoubleFree
    {
        /// The start of the block.
        addr: u64,
    },
    /// A free of an address that is not the start of a heap block.
    BadFree
    {
        /// The address.
        addr: u64,
    },
    /// An access at an address that is not a multiple of the alignment of
    /// its kind.
    Misaligned
    {
        /// The address.
        addr: u64,
        /// The alignment of the kind.
        align: u32,
    },
    /// A load of octets that hold no value of the kind.
    BadValue
    {
        /// The address.
        addr: u64,
        /// The name of the kind.
        kind: String,
    },
    /// The address space had no room for the block of a global.
    StaticFull
    {
        /// The octet count of the global.
        size: u64,
    },
    /// A call found no room in the stack region for a local.
    StackFull
    {
        /// The octet count of the local.
        size: u64,
    },
    /// A resize found no heap room.
    OutOfMem
    {
        /// The octet count asked for.
        size: u64,
    },
    /// A subtraction of pointers into different blocks.
    MixedBlocks,
    /// A cell number outside its list.
    IndexOutOfRange
    {
        /// The cell number.
        index: i128,
        /// The number of the first cell: 0 or 1.
        first: u8,
        /// The number of cells.
        len: u32,
    },
}

impl fmt::Display for TrapKind
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let text = match self
        {
            TrapKind::ZeroDivisor => t!("vm.trap.zero_divisor"),
            TrapKind::ShiftTooFar { distance, bits } =>
            {
                t!("vm.trap.shift_too_far", distance = distance, bits = bits)
            }
            TrapKind::StackOverflow { depth } =>
            {
                t!("vm.trap.stack_overflow", depth = depth)
            }
            TrapKind::StepLimit { steps } => t!("vm.trap.step_limit", steps = steps),
            TrapKind::StepNotPositive => t!("vm.trap.step_not_positive"),
            TrapKind::Malformed { reason } => t!("vm.trap.malformed", reason = reason),
            TrapKind::Write { msg } => t!("vm.trap.write", msg = msg),
            TrapKind::NullDeref { addr } =>
            {
                t!("vm.trap.null_deref", addr = hex(*addr))
            }
            TrapKind::OutOfBounds { addr, size } =>
            {
                t!("vm.trap.out_of_bounds", addr = hex(*addr), size = size)
            }
            TrapKind::UseAfterFree { addr } =>
            {
                t!("vm.trap.use_after_free", addr = hex(*addr))
            }
            TrapKind::DoubleFree { addr } =>
            {
                t!("vm.trap.double_free", addr = hex(*addr))
            }
            TrapKind::BadFree { addr } => t!("vm.trap.bad_free", addr = hex(*addr)),
            TrapKind::Misaligned { addr, align } =>
            {
                t!("vm.trap.misaligned", addr = hex(*addr), align = align)
            }
            TrapKind::BadValue { addr, kind } =>
            {
                t!("vm.trap.bad_value", addr = hex(*addr), kind = kind)
            }
            TrapKind::StaticFull { size } => t!("vm.trap.static_full", size = size),
            TrapKind::StackFull { size } => t!("vm.trap.stack_full", size = size),
            TrapKind::OutOfMem { size } => t!("vm.trap.out_of_mem", size = size),
            TrapKind::MixedBlocks => t!("vm.trap.mixed_blocks"),
            TrapKind::IndexOutOfRange { index, first, len } =>
            {
                let last = i64::from(*len) + i64::from(*first) - 1;
                t!("vm.trap.index_out_of_range", index = index, lo = first, hi = last)
            }
        };
        f.write_str(&text)
    }
}

impl TrapKind
{
    /// The address an access or free trap names.
    ///
    /// # Returns
    ///
    /// The address, or `None` for a trap that names none.
    pub fn addr(&self) -> Option<u64>
    {
        match self
        {
            TrapKind::NullDeref { addr }
            | TrapKind::OutOfBounds { addr, .. }
            | TrapKind::UseAfterFree { addr }
            | TrapKind::DoubleFree { addr }
            | TrapKind::BadFree { addr }
            | TrapKind::Misaligned { addr, .. }
            | TrapKind::BadValue { addr, .. } => Some(*addr),
            TrapKind::ZeroDivisor
            | TrapKind::ShiftTooFar { .. }
            | TrapKind::StackOverflow { .. }
            | TrapKind::StepLimit { .. }
            | TrapKind::StepNotPositive
            | TrapKind::Malformed { .. }
            | TrapKind::Write { .. }
            | TrapKind::StaticFull { .. }
            | TrapKind::StackFull { .. }
            | TrapKind::OutOfMem { .. }
            | TrapKind::MixedBlocks
            | TrapKind::IndexOutOfRange { .. } => None,
        }
    }
}

/// An address as `0x` and lowercase hex digits.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The text.
pub fn hex(addr: u64) -> String { format!("{addr:#x}") }

/// A trap with the place it happened.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct Trap
{
    /// The reason the program stopped.
    pub kind: TrapKind,
    /// The name of the function that ran.
    #[builder(into)]
    pub func: String,
    /// The index of the op that trapped.
    pub pc: u32,
    /// The block the address of the trap falls in, as text: its start,
    /// size and origin.
    pub block: Option<String>,
}

impl fmt::Display for Trap
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let what = self.kind.to_string();
        let text = t!("vm.trap.at", what = what, func = self.func, pc = self.pc);
        f.write_str(&text)?;
        self.block.as_ref().map_or(Ok(()), |b| write!(f, "; {b}"))
    }
}
