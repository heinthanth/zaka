use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::AsRefStr;
use strum::EnumString;
use zakac_lexer::token::DigitScript;
use zakac_lexer::token::Particle;
use zakac_span::span::Span;

use crate::kind::Kind;
use crate::verb::Dir;
use crate::verb::VerbOp;

/// Number of an expression node, unique within one parse. Later passes index
/// side tables with it, since two nodes can share a span.
#[derive(Debug, Clone, Copy, Hash)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// A name written in a declaration, a parameter, a loop header or a call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Ident
{
    /// The name text, without backticks for a raw identifier.
    #[builder(into)]
    pub name: String,
    /// Source region of the name.
    pub span: Span,
}

/// What follows a clause-final verb, a boolean literal or a postfix
/// `မဟုတ်`. The naturalness check picks the idiomatic particle from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextCtx
{
    /// Another token.
    #[default]
    Other,
    /// `သို့မဟုတ်` or `||`
    Or,
    /// `လျှင်`
    Then,
    /// `ဖြစ်လျှင်`
    ThenIs,
    /// `နေသရွေ့`
    While,
    /// `ဖြစ်နေသရွေ့`
    WhileIs,
    /// `ဖြစ်၍` or `&&`
    And,
    /// `)`
    Close,
}

/// An expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expr
{
    /// Integer literal.
    Int(IntLit),
    /// Floating-point literal.
    Float(FloatLit),
    /// Character literal.
    Char(CharLit),
    /// `မှန်` or `မမှန်`.
    Bool(BoolLit),
    /// A variable.
    Name(NameExpr),
    /// `( expr )`
    Group(GroupExpr),
    /// `- အနုတ် ~ !` before an operand.
    Prefix(PrefixExpr),
    /// A notation operator between two operands.
    Infix(InfixExpr),
    /// Postfix `မဟုတ်`.
    Not(NotExpr),
    /// `ဖြစ်၍ သို့မဟုတ် && ||`
    Logic(LogicExpr),
    /// A clause closed with a verb-noun or comparison verb.
    Verb(VerbExpr),
    /// `လုပ်ဆောင်ချက် f ကို ... လုပ်ဆောင်သော ရလဒ်`
    Call(CallExpr),
    /// `ဗလာ`, the address that points nowhere.
    Null(NullLit),
    /// `kind ၏ အရွယ်အစား`
    SizeOf(SizeOfExpr),
    /// `x ၏ အကွက် i` or `x ၏ n ခုမြောက်အကွက်`: one cell of a list.
    Index(IndexExpr),
    /// `a၊ b၊ c တို့ပါသော စာရင်း`
    List(ListLit),
    /// `စာရင်းအလွတ်`, a list with each cell at its default.
    BlankList(BlankListLit),
}

/// Integer literal; the digits are converted with the base.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct IntLit
{
    /// Node number.
    pub id: NodeId,
    /// The value, as an unsigned bit pattern.
    pub value: u128,
    /// The number base, 10 unless a `အခြေ N ဖြင့်` prefix is written.
    pub base: u8,
    /// The kind prefix, when written.
    pub kind: Option<Kind>,
    /// The digit script.
    pub script: DigitScript,
    /// Source region, prefixes included.
    pub span: Span,
}

/// Floating-point literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct FloatLit
{
    /// Node number.
    pub id: NodeId,
    /// The value.
    pub value: f64,
    /// The decimal text in Western digits, fraction and exponent kept, so a
    /// later pass can read it at the exact width of its kind.
    #[builder(into)]
    pub text: String,
    /// The kind prefix, when written.
    pub kind: Option<Kind>,
    /// Source region, prefix included.
    pub span: Span,
}

/// Character literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct CharLit
{
    /// Node number.
    pub id: NodeId,
    /// The decoded character.
    pub value: char,
    /// Source region, quotes included.
    pub span: Span,
}

/// Boolean literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct BoolLit
{
    /// Node number.
    pub id: NodeId,
    /// `true` for `မှန်`.
    pub value: bool,
    /// The sentence-final particle written after the word.
    pub particle: Particle,
    /// What follows the literal.
    #[builder(default)]
    pub next: NextCtx,
    /// Source region.
    pub span: Span,
}

/// `ဗလာ`, the null address.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct NullLit
{
    /// Node number.
    pub id: NodeId,
    /// Source region.
    pub span: Span,
}

/// `kind ၏ အရွယ်အစား`: the size of a kind, in octets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct SizeOfExpr
{
    /// Node number.
    pub id: NodeId,
    /// The measured kind.
    pub kind: Kind,
    /// Source region.
    pub span: Span,
}

/// One cell of a list: `x ၏ အကွက် i` counts from 0, `x ၏ n ခုမြောက်အကွက်`
/// counts from 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct IndexExpr
{
    /// Node number.
    pub id: NodeId,
    /// The list, or a pointer to one.
    #[builder(into)]
    pub list: Box<Expr>,
    /// The cell number.
    #[builder(into)]
    pub index: Box<Expr>,
    /// The number of the first cell: 0 for `အကွက် i`, 1 for
    /// `n ခုမြောက်အကွက်`.
    pub first: u8,
    /// Source region, from the list to the index or the verb.
    pub span: Span,
}

/// `a၊ b၊ c တို့ပါသော စာရင်း`: a list written out one cell at a time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ListLit
{
    /// Node number.
    pub id: NodeId,
    /// The cells, in order.
    #[builder(default)]
    pub cells: Vec<Expr>,
    /// Source region, from the first cell to `စာရင်း`.
    pub span: Span,
}

/// `စာရင်းအလွတ်`: a list whose cells all hold their default; the kind comes
/// from the context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct BlankListLit
{
    /// Node number.
    pub id: NodeId,
    /// Source region.
    pub span: Span,
}

/// A variable, or the implicit result variable `ရလဒ်`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct NameExpr
{
    /// Node number.
    pub id: NodeId,
    /// The name text.
    #[builder(into)]
    pub name: String,
    /// Source region.
    pub span: Span,
}

/// `( expr )`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct GroupExpr
{
    /// Node number.
    pub id: NodeId,
    /// The expression inside.
    #[builder(into)]
    pub inner: Box<Expr>,
    /// Source region, parentheses included.
    pub span: Span,
}

/// Prefix operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr)]
#[serde(rename_all = "snake_case")]
pub enum PrefixOp
{
    /// `-` or `အနုတ်`
    #[strum(serialize = "neg")]
    Neg,
    /// `~`
    #[strum(serialize = "~")]
    BitNot,
    /// `!`
    #[strum(serialize = "!")]
    Not,
    /// `&`
    #[strum(serialize = "addr")]
    AddrOf,
    /// `*`
    #[strum(serialize = "deref")]
    Deref,
}

impl PrefixOp
{
    /// The short name of the operator, as asm would write it.
    ///
    /// # Returns
    ///
    /// `neg`, `bnot`, `not`, `addr` or `deref`.
    pub const fn mnemonic(self) -> &'static str
    {
        match self
        {
            PrefixOp::Neg => "neg",
            PrefixOp::BitNot => "bnot",
            PrefixOp::Not => "not",
            PrefixOp::AddrOf => "addr",
            PrefixOp::Deref => "deref",
        }
    }
}

/// A prefix operator applied to an operand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct PrefixExpr
{
    /// Node number.
    pub id: NodeId,
    /// The operator.
    pub op: PrefixOp,
    /// The operand.
    #[builder(into)]
    pub rhs: Box<Expr>,
    /// Source region.
    pub span: Span,
}

/// Infix notation operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr, EnumString)]
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
    /// `/`
    #[strum(serialize = "/")]
    Div,
    /// `%`
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
    /// `>>`
    #[strum(serialize = ">>")]
    Shr,
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

impl InfixOp
{
    /// The short name of the operator, as asm would write it.
    ///
    /// # Returns
    ///
    /// The name, such as `add` for `+` or `lt` for `<`.
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
            InfixOp::Eq => "eq",
            InfixOp::Ne => "ne",
            InfixOp::Lt => "lt",
            InfixOp::Gt => "gt",
            InfixOp::Le => "le",
            InfixOp::Ge => "ge",
        }
    }

    /// Whether this operator is a comparison.
    ///
    /// # Returns
    ///
    /// `true` for `== != < > <= >=`.
    pub const fn is_cmp(self) -> bool
    {
        matches!(
            self,
            InfixOp::Eq
                | InfixOp::Ne
                | InfixOp::Lt
                | InfixOp::Gt
                | InfixOp::Le
                | InfixOp::Ge
        )
    }
}

/// A notation operator between two operands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct InfixExpr
{
    /// Node number.
    pub id: NodeId,
    /// The operator.
    pub op: InfixOp,
    /// The left operand.
    #[builder(into)]
    pub lhs: Box<Expr>,
    /// The right operand.
    #[builder(into)]
    pub rhs: Box<Expr>,
    /// Source region.
    pub span: Span,
}

/// Postfix `မဟုတ်`, applied to a whole clause.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct NotExpr
{
    /// Node number.
    pub id: NodeId,
    /// The negated clause.
    #[builder(into)]
    pub inner: Box<Expr>,
    /// The sentence-final particle written after `မဟုတ်`.
    pub particle: Particle,
    /// What follows `မဟုတ်`.
    #[builder(default)]
    pub next: NextCtx,
    /// Source region.
    pub span: Span,
}

/// Logical operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LogicOp
{
    /// `ဖြစ်၍` or `&&`
    And,
    /// `သို့မဟုတ်` or `||`
    Or,
}

/// A logical operator between two clauses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct LogicExpr
{
    /// Node number.
    pub id: NodeId,
    /// The operator.
    pub op: LogicOp,
    /// The left clause.
    #[builder(into)]
    pub lhs: Box<Expr>,
    /// The right clause.
    #[builder(into)]
    pub rhs: Box<Expr>,
    /// Source region.
    pub span: Span,
}

/// A clause closed with a verb-noun or a comparison verb, such as
/// `က နှင့် ခ ၏ ပေါင်းလဒ်`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct VerbExpr
{
    /// Node number.
    pub id: NodeId,
    /// The operation.
    pub op: VerbOp,
    /// The expression operands, in source order.
    #[builder(default)]
    pub args: Vec<Expr>,
    /// The target kind of a conversion.
    pub kind_arg: Option<Kind>,
    /// The direction of a shift.
    pub dir: Option<Dir>,
    /// The sentence-final particle written after the verb.
    pub particle: Particle,
    /// What follows the verb.
    #[builder(default)]
    pub next: NextCtx,
    /// Whether the verb is a comparison.
    pub is_cmp: bool,
    /// Source region, from the first operand to the verb.
    pub span: Span,
}

/// `လုပ်ဆောင်ချက် f ကို a၊ b တို့ဖြင့် လုပ်ဆောင်သော ရလဒ်`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct CallExpr
{
    /// Node number.
    pub id: NodeId,
    /// The function name.
    pub callee: Ident,
    /// The arguments.
    #[builder(default)]
    pub args: Vec<Expr>,
    /// Source region.
    pub span: Span,
}

impl Expr
{
    /// The node number.
    ///
    /// # Returns
    ///
    /// The number of the wrapped node.
    pub fn id(&self) -> NodeId
    {
        match self
        {
            Expr::Int(n) => n.id,
            Expr::Float(n) => n.id,
            Expr::Char(n) => n.id,
            Expr::Bool(n) => n.id,
            Expr::Name(n) => n.id,
            Expr::Group(n) => n.id,
            Expr::Prefix(n) => n.id,
            Expr::Infix(n) => n.id,
            Expr::Not(n) => n.id,
            Expr::Logic(n) => n.id,
            Expr::Verb(n) => n.id,
            Expr::Call(n) => n.id,
            Expr::Null(n) => n.id,
            Expr::SizeOf(n) => n.id,
            Expr::Index(n) => n.id,
            Expr::List(n) => n.id,
            Expr::BlankList(n) => n.id,
        }
    }

    /// Whether this expression is a comparison that is not in parentheses,
    /// written either with a notation operator or with a verb.
    ///
    /// # Returns
    ///
    /// `true` for an unparenthesized comparison.
    pub fn is_cmp(&self) -> bool
    {
        match self
        {
            Expr::Infix(n) => n.op.is_cmp(),
            Expr::Verb(n) => n.is_cmp,
            _ => false,
        }
    }

    /// Whether this expression is a place that can be written or have its
    /// address taken: a variable, a dereference, a list cell, or one of
    /// these in parentheses.
    ///
    /// # Returns
    ///
    /// `true` for a place.
    pub fn is_place(&self) -> bool
    {
        match self
        {
            Expr::Name(_) => true,
            Expr::Group(n) => n.inner.is_place(),
            Expr::Prefix(n) => n.op == PrefixOp::Deref,
            Expr::Verb(n) => n.op == VerbOp::Deref,
            Expr::Index(_) => true,
            _ => false,
        }
    }
}
