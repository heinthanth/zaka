use std::fmt;
use std::str::FromStr;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoStaticStr;
use zakac_ast::stmt::CountMode;
use zakac_semcheck::kind::DataKind;
use zakac_span::span::Span;

use crate::value::Value;

/// A lowered program: the global slots, the functions and the top-level
/// statements, which run in order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Module
{
    /// Variables declared outside a function, in declaration order.
    #[builder(default)]
    pub globals: Vec<Slot>,
    /// Functions, in source order; a call names one with its index.
    #[builder(default)]
    pub funcs: Vec<Func>,
    /// The top-level statements.
    #[builder(default)]
    pub main: Vec<Stmt>,
    /// The number of statements at the start of `main` that set the
    /// variables and constants of imported files. Imported files hold no
    /// other statements, and the file that is run comes after them.
    #[builder(default)]
    #[serde(default)]
    pub imported_inits: u32,
    /// Source region of the whole program.
    pub span: Span,
}

/// A function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Func
{
    /// The function name.
    #[builder(into)]
    pub name: String,
    /// The number of parameters; `locals[..params]` are the parameters.
    pub params: u32,
    /// The local slot of the implicit result variable, for a function with
    /// a result.
    pub result: Option<u32>,
    /// The result kind, for a function with a result.
    pub ret: Option<DataKind>,
    /// Parameters, the result variable, then the other locals, in
    /// declaration order.
    #[builder(default)]
    pub locals: Vec<Slot>,
    /// The statements.
    #[builder(default)]
    pub block: Vec<Stmt>,
    /// Source region of the declaration.
    pub span: Span,
}

/// What a slot allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Access
{
    /// `ကိန်းရှင်`, a parameter or the result variable.
    Var,
    /// `ကိန်းသေ`: set once, where it is declared.
    Const,
    /// The variable of a counted loop.
    LoopVar,
}

/// Storage for one declared name. Each declaration gets its own slot, so a
/// shadowing name never shares one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Slot
{
    /// The declared name.
    #[builder(into)]
    pub name: String,
    /// The data kind.
    pub kind: DataKind,
    /// What the slot allows.
    pub access: Access,
    /// Source region of the name.
    pub span: Span,
    /// Whether the program takes the address of the slot, so it needs a
    /// fixed address.
    #[serde(default)]
    #[builder(default)]
    pub addressed: bool,
}

/// Where a slot lives. Written `g0` for a global and `l0` for a local of the
/// current function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Place
{
    /// An index into [`Module::globals`].
    Global(u32),
    /// An index into [`Func::locals`] of the running function.
    Local(u32),
}

impl fmt::Display for Place
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Place::Global(n) => write!(f, "g{n}"),
            Place::Local(n) => write!(f, "l{n}"),
        }
    }
}

impl FromStr for Place
{
    type Err = String;

    /// Read `g0` or `l0`.
    ///
    /// # Arguments
    ///
    /// * `text` - The text.
    ///
    /// # Returns
    ///
    /// The place, or an error for other text.
    fn from_str(text: &str) -> Result<Self, Self::Err>
    {
        let bad = || format!("unknown place {text}");
        let (head, digits) = text.split_at_checked(1).ok_or_else(bad)?;
        let n: u32 = digits.parse().map_err(|_| bad())?;
        match head
        {
            "g" => Ok(Place::Global(n)),
            "l" => Ok(Place::Local(n)),
            _ => Err(bad()),
        }
    }
}

impl From<Place> for String
{
    fn from(place: Place) -> String { place.to_string() }
}

impl TryFrom<String> for Place
{
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> { text.parse() }
}

/// Where a heap statement keeps the address it works on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dest
{
    /// A slot of a pointer kind.
    Slot(Place),
    /// The cell at an address, which holds a pointer.
    Mem(Expr),
}

/// What a print statement writes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintValue
{
    /// A value.
    Expr(Expr),
    /// A string literal, decoded.
    Text(String),
}

/// A statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stmt
{
    /// Store a value: a declaration with its initial value, or an
    /// assignment.
    Set
    {
        /// The slot.
        place: Place,
        /// The value, of the slot's kind.
        value: Expr,
        /// Source region.
        span: Span,
    },
    /// Write a value and a line break.
    Print
    {
        /// What to write.
        value: PrintValue,
        /// Source region.
        span: Span,
    },
    /// Run `then` when `cond` holds, else `els`. An `else if` chain nests in
    /// `els`.
    If
    {
        /// The condition, a boolean.
        cond: Expr,
        /// Statements when it holds.
        then: Vec<Stmt>,
        /// Statements when it does not.
        els: Vec<Stmt>,
        /// Source region.
        span: Span,
    },
    /// Run `init` once, then repeat: test `cond`, run `block`, run
    /// `update`. `continue` jumps to `update`. A while loop has no `init`
    /// and no `update`.
    Loop
    {
        /// Statements before the first test.
        init: Vec<Stmt>,
        /// The condition, a boolean.
        cond: Expr,
        /// Statements after each round.
        update: Vec<Stmt>,
        /// The loop statements.
        block: Vec<Stmt>,
        /// Source region.
        span: Span,
    },
    /// Counted loop: `var` runs from `from` to `to`, both included.
    Counted
    {
        /// The loop variable.
        var: Place,
        /// The first value, of the variable's kind.
        from: Expr,
        /// The last value, of the variable's kind.
        to: Expr,
        /// The step, of the variable's kind; one when absent.
        step: Option<Expr>,
        /// Which direction the variable moves.
        mode: CountMode,
        /// The loop statements.
        block: Vec<Stmt>,
        /// Source region.
        span: Span,
    },
    /// Leave the innermost loop.
    Break
    {
        /// Source region.
        span: Span,
    },
    /// Go on with the next round of the innermost loop.
    Continue
    {
        /// Source region.
        span: Span,
    },
    /// Leave the function, with a value when it has a result.
    Return
    {
        /// The value, of the result kind.
        value: Option<Expr>,
        /// Source region.
        span: Span,
    },
    /// Call a function and drop its result.
    Call
    {
        /// Index into [`Module::funcs`].
        func: u32,
        /// The function name.
        name: String,
        /// The arguments, each of its parameter's kind.
        args: Vec<Expr>,
        /// Source region.
        span: Span,
    },
    /// Write a value at an address.
    Store
    {
        /// The address, a pointer.
        addr: Expr,
        /// The value, of the kind the address points at.
        value: Expr,
        /// Source region.
        span: Span,
    },
    /// Take a new heap block and keep its address in `dest`.
    Alloc
    {
        /// Where the address goes.
        dest: Dest,
        /// The element count, `usize`; the size in octets without a unit.
        count: Expr,
        /// The element kind, when written.
        unit: Option<DataKind>,
        /// Source region.
        span: Span,
    },
    /// Give the heap block at `dest` a new size and keep its new address
    /// there.
    Resize
    {
        /// Where the address lives.
        dest: Dest,
        /// The element count, `usize`; the size in octets without a unit.
        count: Expr,
        /// The element kind, when written.
        unit: Option<DataKind>,
        /// Source region.
        span: Span,
    },
    /// Release the heap block at `dest`.
    Free
    {
        /// Where the address lives.
        dest: Dest,
        /// Source region.
        span: Span,
    },
}

impl Stmt
{
    /// The source region.
    ///
    /// # Returns
    ///
    /// The span.
    pub fn span(&self) -> Span
    {
        match self
        {
            Stmt::Set { span, .. }
            | Stmt::Print { span, .. }
            | Stmt::If { span, .. }
            | Stmt::Loop { span, .. }
            | Stmt::Counted { span, .. }
            | Stmt::Break { span }
            | Stmt::Continue { span }
            | Stmt::Return { span, .. }
            | Stmt::Call { span, .. }
            | Stmt::Store { span, .. }
            | Stmt::Alloc { span, .. }
            | Stmt::Resize { span, .. }
            | Stmt::Free { span, .. } => *span,
        }
    }
}

/// An expression with its data kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Expr
{
    /// The kind of the value.
    pub kind: DataKind,
    /// The operation.
    pub node: ExprNode,
    /// Source region.
    pub span: Span,
}

impl Expr
{
    /// The value, when the expression is a constant.
    ///
    /// # Returns
    ///
    /// The value, or `None`.
    pub fn as_const(&self) -> Option<Value>
    {
        match self.node
        {
            ExprNode::Const(v) => Some(v),
            _ => None,
        }
    }
}

/// An operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExprNode
{
    /// A constant.
    Const(Value),
    /// The value of a slot.
    Read(Place),
    /// A prefix operator; the operand has the result kind.
    Prefix
    {
        /// The operator.
        op: PrefixOp,
        /// The operand.
        operand: Box<Expr>,
    },
    /// Arithmetic, bitwise and shift operators. Both operands have the
    /// result kind, except a shift distance, which is an unsigned integer
    /// of its own kind.
    Infix
    {
        /// The operator.
        op: InfixOp,
        /// The left operand.
        lhs: Box<Expr>,
        /// The right operand.
        rhs: Box<Expr>,
    },
    /// A comparison; both operands have one common kind.
    Compare
    {
        /// The operator.
        op: CmpOp,
        /// The left operand.
        lhs: Box<Expr>,
        /// The right operand.
        rhs: Box<Expr>,
    },
    /// `and` or `or`; the right operand runs just when needed.
    Logic
    {
        /// The operator.
        op: LogicOp,
        /// The left operand.
        lhs: Box<Expr>,
        /// The right operand.
        rhs: Box<Expr>,
    },
    /// A conversion to the expression kind.
    Convert
    {
        /// The value.
        operand: Box<Expr>,
        /// `true` for a widening the compiler added, `false` for a written
        /// conversion.
        implicit: bool,
    },
    /// A function call.
    Call
    {
        /// Index into [`Module::funcs`].
        func: u32,
        /// The function name.
        name: String,
        /// The arguments, each of its parameter's kind.
        args: Vec<Expr>,
    },
    /// The address of a slot. A slot whose address the program takes is
    /// marked `addressed`; the address a [`ExprNode::Take`] moves through
    /// leaves the mark alone.
    AddrOf(Place),
    /// The value at an address; the expression kind is the pointee.
    Load
    {
        /// The address, a pointer.
        addr: Box<Expr>,
    },
    /// A borrow of an address: the value of `operand` under a borrow mark.
    Lend
    {
        /// The address lent, a pointer.
        operand: Box<Expr>,
    },
    /// The value at an address, moved out: the place is left at the
    /// default of its kind, `ဗလာ` for an owner. The expression kind is the
    /// kind of the place.
    Take
    {
        /// The address of the place, a pointer.
        addr: Box<Expr>,
    },
    /// An address moved `offset` elements forward; the expression kind is
    /// the kind of `base`.
    PtrAdd
    {
        /// The address, a pointer.
        base: Box<Expr>,
        /// The element count, an integer of its own kind.
        offset: Box<Expr>,
    },
    /// The element count between two addresses of one kind, an `isize`.
    PtrSub
    {
        /// The later address.
        lhs: Box<Expr>,
        /// The earlier address.
        rhs: Box<Expr>,
    },
    /// The size of a kind in octets, a `usize`.
    SizeOf(DataKind),
    /// A list or a struct written out one cell or field at a time, in order;
    /// each one has its cell or field kind in the expression kind.
    List(Vec<Expr>),
    /// A list or a struct of the expression kind with each cell or field at
    /// its default.
    BlankList,
    /// The address of one cell of a list; the expression kind is a pointer
    /// to the cell kind.
    CellAddr
    {
        /// The list, or a pointer to it.
        list: Box<Expr>,
        /// The cell number, an integer of its own kind.
        index: Box<Expr>,
        /// The number of cells.
        len: u32,
        /// The number of the first cell: 0 or 1.
        first: u8,
        /// Whether the cell number is known to be in range, so no check is
        /// needed when the program runs.
        checked: bool,
    },
    /// The address of one field of a struct; the expression kind is a
    /// pointer to the field kind.
    FieldAddr
    {
        /// The struct.
        base: Box<Expr>,
        /// The position of the field in the struct.
        field: u32,
        /// The offset of the field in the struct, in octets.
        offset: u32,
    },
}

/// A prefix operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(IntoStaticStr)]
#[serde(rename_all = "snake_case")]
pub enum PrefixOp
{
    /// Negate a signed integer or a float.
    #[strum(serialize = "neg")]
    Neg,
    /// Flip the bits of an integer.
    #[strum(serialize = "~")]
    BitNot,
    /// Boolean not.
    #[strum(serialize = "!")]
    Not,
}

impl PrefixOp
{
    /// The short name of the operator, as asm would write it.
    ///
    /// # Returns
    ///
    /// `neg`, `bnot` or `not`.
    pub const fn mnemonic(self) -> &'static str
    {
        match self
        {
            PrefixOp::Neg => "neg",
            PrefixOp::BitNot => "bnot",
            PrefixOp::Not => "not",
        }
    }
}

/// An arithmetic, bitwise or shift operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(IntoStaticStr)]
#[serde(rename_all = "snake_case")]
pub enum InfixOp
{
    /// `+`
    #[strum(serialize = "+")]
    Add,
    /// `-`
    #[strum(serialize = "-")]
    Sub,
    /// `*`
    #[strum(serialize = "*")]
    Mul,
    /// `/`, toward zero for integers.
    #[strum(serialize = "/")]
    Div,
    /// `%`, with the sign of the dividend.
    #[strum(serialize = "%")]
    Rem,
    /// `&`
    #[strum(serialize = "&")]
    BitAnd,
    /// `|`
    #[strum(serialize = "|")]
    BitOr,
    /// `^`
    #[strum(serialize = "^")]
    BitXor,
    /// `<<`
    #[strum(serialize = "<<")]
    Shl,
    /// `>>`, arithmetic for signed integers.
    #[strum(serialize = ">>")]
    Shr,
}

impl InfixOp
{
    /// The short name of the operator, as asm would write it.
    ///
    /// # Returns
    ///
    /// The name, such as `add` for `+` or `shl` for `<<`.
    pub const fn mnemonic(self) -> &'static str
    {
        match self
        {
            InfixOp::Add => "add",
            InfixOp::Sub => "sub",
            InfixOp::Mul => "mul",
            InfixOp::Div => "div",
            InfixOp::Rem => "rem",
            InfixOp::BitAnd => "band",
            InfixOp::BitOr => "bor",
            InfixOp::BitXor => "bxor",
            InfixOp::Shl => "shl",
            InfixOp::Shr => "shr",
        }
    }
}

/// A comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(IntoStaticStr)]
#[serde(rename_all = "snake_case")]
pub enum CmpOp
{
    /// `==`
    #[strum(serialize = "==")]
    Eq,
    /// `!=`
    #[strum(serialize = "!=")]
    Ne,
    /// `<`
    #[strum(serialize = "<")]
    Lt,
    /// `>`
    #[strum(serialize = ">")]
    Gt,
    /// `<=`
    #[strum(serialize = "<=")]
    Le,
    /// `>=`
    #[strum(serialize = ">=")]
    Ge,
}

impl CmpOp
{
    /// The short name of the operator, as asm would write it.
    ///
    /// # Returns
    ///
    /// The name, such as `eq` for `==` or `le` for `<=`.
    pub const fn mnemonic(self) -> &'static str
    {
        match self
        {
            CmpOp::Eq => "eq",
            CmpOp::Ne => "ne",
            CmpOp::Lt => "lt",
            CmpOp::Gt => "gt",
            CmpOp::Le => "le",
            CmpOp::Ge => "ge",
        }
    }
}

/// A logical operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LogicOp
{
    /// Both hold.
    And,
    /// One of the two holds.
    Or,
}

/// Options of the lowering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder)]
pub struct LowerOptions
{
    /// Bit count of the platform-sized integer kinds.
    #[builder(default = 64)]
    pub platform_bits: u16,
}

impl Default for LowerOptions
{
    /// A 64-bit platform.
    ///
    /// # Returns
    ///
    /// The options.
    fn default() -> Self { LowerOptions::builder().build() }
}

/// Which foldings run. Errors in constant operations are reported
/// whatever these hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Builder)]
pub struct FoldOptions
{
    /// Replace operations on constants with their value.
    #[builder(default = true)]
    pub exprs: bool,
    /// Replace reads of a `ကိန်းသေ` set to a constant with its value.
    #[builder(default = true)]
    pub consts: bool,
    /// Keep the taken branch of an `if` on a constant, and drop a loop
    /// whose condition is the constant false.
    #[builder(default = true)]
    pub branches: bool,
}

impl FoldOptions
{
    /// All foldings off; the errors are still reported.
    ///
    /// # Returns
    ///
    /// The options.
    pub fn none() -> Self
    {
        FoldOptions::builder().exprs(false).consts(false).branches(false).build()
    }
}

impl Default for FoldOptions
{
    /// All foldings on.
    ///
    /// # Returns
    ///
    /// The options.
    fn default() -> Self { FoldOptions::builder().build() }
}
