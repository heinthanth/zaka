//! The operators of constant values: what the folder and the check of a
//! list count work out, and what the IR writes.

use serde::Deserialize;
use serde::Serialize;
use strum::IntoStaticStr;

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
