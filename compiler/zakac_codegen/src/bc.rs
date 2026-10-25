use std::fmt;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoStaticStr;
use zakac_ir_builder::ir::CmpOp;
use zakac_ir_builder::ir::InfixOp;
use zakac_ir_builder::ir::PrefixOp;
use zakac_ir_builder::value::Value;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

/// A program as stack machine ops for the Zaka VM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Module
{
    /// Bit count of the platform-sized integer kinds.
    pub platform_bits: u16,
    /// The global slots.
    #[builder(default)]
    pub globals: Vec<Slot>,
    /// Pooled print strings, indexed with [`Op::PrintText`].
    #[builder(default)]
    pub texts: Vec<String>,
    /// The functions in source order, then the main function.
    #[builder(default)]
    pub funcs: Vec<Func>,
    /// Index of the main function in `funcs`.
    pub main: u32,
}

impl Module
{
    /// Move the source region of each op.
    ///
    /// # Arguments
    ///
    /// * `f` - Maps a region to its new place.
    pub fn map_spans(&mut self, f: &impl Fn(Span) -> Span)
    {
        let spans = self.funcs.iter_mut().flat_map(|func| func.spans.iter_mut());
        spans.for_each(|s| *s = f(*s));
    }
}

/// A named slot of one data kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Slot
{
    /// The source name, or a `%` name for a slot the generator added.
    #[builder(into)]
    pub name: String,
    /// The data kind.
    pub kind: DataKind,
}

/// A function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Func
{
    /// The source name.
    #[builder(into)]
    pub name: String,
    /// The number of parameters; `locals[..params]` hold them.
    pub params: u32,
    /// The result kind, if the function has one.
    pub ret: Option<DataKind>,
    /// Parameters, source locals, then slots the generator added.
    #[builder(default)]
    pub locals: Vec<Slot>,
    /// The ops.
    #[builder(default)]
    pub code: Vec<Op>,
    /// The source region of each op, parallel to `code`; a placeholder for an
    /// op with no source.
    #[builder(default)]
    pub spans: Vec<Span>,
}

/// The reason a [`Op::Trap`] stops the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TrapCode
{
    /// A counted loop step is zero or negative.
    StepNotPositive,
}

/// A stack machine op. Ops that work on values read the kind from their
/// operands; the stack holds kinded values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Op
{
    /// Push a constant.
    Push(Value),
    /// Push the value of a local slot.
    LoadLocal(u32),
    /// Pop a value into a local slot.
    StoreLocal(u32),
    /// Push the value of a global slot.
    LoadGlobal(u32),
    /// Pop a value into a global slot.
    StoreGlobal(u32),
    /// Pop one value, push the result of a prefix operator.
    Prefix(PrefixOp),
    /// Pop the right then the left operand, push the result.
    Infix(InfixOp),
    /// Pop the right then the left operand, push a boolean.
    Compare(CmpOp),
    /// Pop a value, push it converted to this kind.
    Convert(DataKind),
    /// Go to the op at this index of the function.
    Jump(u32),
    /// Pop a boolean; go to the op at this index when it is false.
    JumpIfFalse(u32),
    /// Pop a boolean; go to the op at this index when it is true.
    JumpIfTrue(u32),
    /// Call the function at this index of [`Module::funcs`]; the arguments
    /// are on the stack, the last one on top.
    Call(u32),
    /// Leave the function. A function with a result leaves it on the stack.
    Ret,
    /// Pop a value and write it with a line break.
    Print,
    /// Write a pooled text with a line break.
    PrintText(u32),
    /// Drop the top value.
    Pop,
    /// Stop the program with an error.
    Trap(TrapCode),
}

impl Op
{
    /// The target of a jump.
    ///
    /// # Returns
    ///
    /// The op index, or `None` for an op that does not jump.
    pub fn target(self) -> Option<u32>
    {
        match self
        {
            Op::Jump(t) | Op::JumpIfFalse(t) | Op::JumpIfTrue(t) => Some(t),
            _ => None,
        }
    }

    /// The same jump with another target.
    ///
    /// # Arguments
    ///
    /// * `to` - The new target.
    ///
    /// # Returns
    ///
    /// The op; an op that does not jump comes back unchanged.
    pub fn retarget(self, to: u32) -> Op
    {
        match self
        {
            Op::Jump(_) => Op::Jump(to),
            Op::JumpIfFalse(_) => Op::JumpIfFalse(to),
            Op::JumpIfTrue(_) => Op::JumpIfTrue(to),
            other => other,
        }
    }

    /// The op name, such as `load_local`; an operator op goes by the short
    /// name of its operator, such as `add` or `lt`.
    ///
    /// # Returns
    ///
    /// The name.
    pub fn name(self) -> &'static str
    {
        match self
        {
            Op::Prefix(op) => op.mnemonic(),
            Op::Infix(op) => op.mnemonic(),
            Op::Compare(op) => op.mnemonic(),
            _ => self.into(),
        }
    }

    /// The operand as listing text, such as `l3` or `0:isize`.
    ///
    /// # Returns
    ///
    /// The text, or `None` for an op without an operand.
    pub fn operand(self) -> Option<String>
    {
        let text = match self
        {
            Op::Push(v) => v.to_string(),
            Op::LoadLocal(n) | Op::StoreLocal(n) => format!("l{n}"),
            Op::LoadGlobal(n) | Op::StoreGlobal(n) => format!("g{n}"),
            Op::Convert(kind) => kind.name(),
            Op::Jump(t) | Op::JumpIfFalse(t) | Op::JumpIfTrue(t) => format!("@{t}"),
            Op::Call(f) => format!("f{f}"),
            Op::PrintText(t) => format!("t{t}"),
            Op::Trap(code) => <&str>::from(code).to_string(),
            Op::Ret | Op::Print | Op::Pop => return None,
            Op::Prefix(_) | Op::Infix(_) | Op::Compare(_) => return None,
        };
        Some(text)
    }
}

impl fmt::Display for Op
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let name = self.name();
        match self.operand()
        {
            Some(operand) => write!(f, "{name} {operand}"),
            None => f.write_str(name),
        }
    }
}
