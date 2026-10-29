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
    /// Push the address of the block of a local slot of the running call.
    LocalAddr(u32),
    /// Push the address of the block of a global slot.
    GlobalAddr(u32),
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
    /// Pop an address, push the value of this kind at it.
    Load(DataKind),
    /// Pop a value of this kind, then an address; write the value there.
    Store(DataKind),
    /// Pop an address; push the value of this kind at it and leave the
    /// default of the kind there, `ဗလာ` for an owner.
    Take(DataKind),
    /// Pop an integer count, then an address; push the address moved that
    /// count of elements of this stride forward.
    PtrAdd(u32),
    /// Pop two addresses; push the element count between them, an `isize`,
    /// for elements of this stride.
    PtrSub(u32),
    /// Pop an element count; push the address of a new heap block for that
    /// count of elements of this stride, or null when none is left.
    Alloc(u32),
    /// Pop an element count, then the address of a heap block; push the
    /// address of the block resized to hold that count of elements of this
    /// stride.
    Realloc(u32),
    /// Pop the address of a heap block and release it.
    Free,
    /// Push the top value a second time.
    Dup,
    /// Pop a cell number; stop the program when it is outside the list,
    /// else push it counted from 0.
    Index
    {
        /// The number of cells.
        len: u32,
        /// The number of the first cell: 0 or 1.
        first: u8,
    },
    /// Pop a source address, then a destination address; duplicate a value of
    /// this kind, such as a whole list, from the source to the destination.
    Blit(DataKind),
    /// Pop an address; set the octets of a value of this kind there to
    /// zero.
    Zero(DataKind),
    /// Pop the addresses of two lists of this kind, the right one first;
    /// push whether each cell of one equals the cell of the other.
    ListEq(DataKind),
    /// Pop the address of a list of this kind; write its cells and a line
    /// break.
    PrintList(DataKind),
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
            Op::LocalAddr(n) => format!("l{n}"),
            Op::GlobalAddr(n) => format!("g{n}"),
            Op::Convert(kind) => kind.name(),
            Op::Jump(t) | Op::JumpIfFalse(t) | Op::JumpIfTrue(t) => format!("@{t}"),
            Op::Call(f) => format!("f{f}"),
            Op::PrintText(t) => format!("t{t}"),
            Op::Trap(code) => <&str>::from(code).to_string(),
            Op::Load(kind)
            | Op::Store(kind)
            | Op::Take(kind)
            | Op::Blit(kind)
            | Op::Zero(kind)
            | Op::ListEq(kind)
            | Op::PrintList(kind) => kind.name(),
            Op::Index { len, first: 0 } => len.to_string(),
            Op::Index { len, first } => format!("{len} from {first}"),
            Op::PtrAdd(s) | Op::PtrSub(s) | Op::Alloc(s) | Op::Realloc(s) =>
            {
                s.to_string()
            }
            Op::Ret | Op::Print | Op::Pop | Op::Free | Op::Dup => return None,
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
