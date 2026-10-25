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
        };
        f.write_str(&text)
    }
}

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
}

impl fmt::Display for Trap
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let what = self.kind.to_string();
        let text = t!("vm.trap.at", what = what, func = self.func, pc = self.pc);
        f.write_str(&text)
    }
}
