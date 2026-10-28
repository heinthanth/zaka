use serde::Deserialize;
use serde::Serialize;
use strum::AsRefStr;
use zakac_lexer::token::Kw;
use zakac_rust_sugars::ret_true;

/// Direction of a shift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dir
{
    /// `ဘယ်သို့`
    Left,
    /// `ညာသို့`
    Right,
}

/// The particle an operand must hold to fill a verb's role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr)]
#[serde(rename_all = "snake_case")]
pub enum Slot
{
    /// `နှင့်`
    #[strum(serialize = "နှင့်")]
    Hnint,
    /// `၏`
    #[strum(serialize = "၏")]
    Ei,
    /// `ကို`
    #[strum(serialize = "ကို")]
    Ko,
    /// `ဖြင့်`
    #[strum(serialize = "ဖြင့်")]
    Hpjint,
    /// `သည်`
    #[strum(serialize = "သည်")]
    Thi,
    /// `ထက်`
    #[strum(serialize = "ထက်")]
    Htet,
    /// `အောက်`
    #[strum(serialize = "အောက်")]
    Auk,
    /// `ဟု`
    #[strum(serialize = "ဟု")]
    Hu,
    /// `ဘယ်သို့|ညာသို့ <operand> ဘစ်`; a verb that lists one direction
    /// accepts both.
    #[strum(serialize = "ဘယ်သို့|ညာသို့ ... ဘစ်")]
    Bit(Dir),
    /// `ကို` or `ဟု`; listed as a wanted slot, never carried.
    #[strum(serialize = "ကို|ဟု")]
    KoOrHu,
    /// `<kind> အဖြစ်`
    #[strum(serialize = "... အဖြစ်")]
    KindAhpjit,
    /// `ခုမြောက်`, after the number of a cell counted from 1.
    #[strum(serialize = "ခုမြောက်")]
    Ordinal,
}

impl Slot
{
    /// Whether an operand that carries `self` fills a role that wants
    /// `want`.
    ///
    /// # Arguments
    ///
    /// * `want` - The slot the verb lists.
    ///
    /// # Returns
    ///
    /// `true` when the two match; shift slots match in either direction, and
    /// `ကို` or `ဟု` fills a slot that lists both.
    pub fn fills(self, want: Slot) -> bool
    {
        ret_true!(matches!((self, want), (Slot::Bit(_), Slot::Bit(_))));
        ret_true!(matches!((self, want), (Slot::Ko | Slot::Hu, Slot::KoOrHu)));
        self == want
    }
}

/// The operation a verb performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "lowercase")]
pub enum VerbOp
{
    /// `ပေါင်းလဒ်`
    Add,
    /// `နုတ်လဒ်`
    Sub,
    /// `မြှောက်လဒ်`
    Mul,
    /// `စားသော စားလဒ်`
    Div,
    /// `စားသော စားကြွင်း`
    Rem,
    /// `ဘစ်အလိုက် ဆုံလဒ်`
    BitAnd,
    /// `ဘစ်အလိုက် ပေါင်းစုလဒ်`
    BitOr,
    /// `ဘစ်အလိုက် ခြားနားလဒ်`
    BitXor,
    /// `ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်`
    BitNot,
    /// `ရွေ့သောရလဒ်`
    Shift,
    /// `ညီ`
    Eq,
    /// `မညီ`
    Ne,
    /// `ကြီး`
    Gt,
    /// `မကြီး`
    Le,
    /// `ငယ်`
    Lt,
    /// `မငယ်`
    Ge,
    /// `ပြောင်းလဲသော ရလဒ်`
    Cast,
    /// `ပြသပါ`
    Print,
    /// `ထားပါ`
    #[strum(serialize = "set")]
    Assign,
    /// `ရလဒ်အဖြစ် ပြန်ပေးပါ`
    Return,
    /// `၏ လိပ်စာ`: the address of a place.
    #[strum(serialize = "addr")]
    AddrOf,
    /// `၏ တန်ဖိုး`: the value an address points at.
    #[strum(serialize = "deref")]
    Deref,
    /// `၏ n ခုမြောက်အကွက်`: a list cell counted from 1. The parser turns
    /// it into an index expression.
    #[strum(serialize = "cell1")]
    Cell,
    /// `၏ အကွက်အရေအတွက်`: the number of cells of a list.
    #[strum(serialize = "cell_count")]
    CellCount,
}

/// One row of the valence table: a verb, the particles its operands hold,
/// and whether it ends a statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verb
{
    /// The verb token.
    pub kw: Kw,
    /// The operation.
    pub op: VerbOp,
    /// The slot of each operand, in source order.
    pub slots: &'static [Slot],
    /// Whether the verb closes a statement instead of producing a value.
    pub is_stmt: bool,
    /// Whether the verb is a comparison, which cannot take a comparison as
    /// a direct operand.
    pub is_cmp: bool,
}

/// Make a valence table row.
#[rustfmt::skip]
const fn row(kw: Kw, op: VerbOp, slots: &'static [Slot], is_stmt: bool, is_cmp: bool) -> Verb
{
    Verb { kw, op, slots, is_stmt, is_cmp }
}

const PAIR: &[Slot] = &[Slot::Hnint, Slot::Ei];
const DIVIDE: &[Slot] = &[Slot::Ko, Slot::Hpjint];
const EQUAL: &[Slot] = &[Slot::Thi, Slot::Hnint];
const GREATER: &[Slot] = &[Slot::Thi, Slot::Htet];
const LESS: &[Slot] = &[Slot::Thi, Slot::Auk];

/// The valence table: each verb the clause engine reduces on.
#[rustfmt::skip]
pub static VERBS: &[Verb] = &[
    row(Kw::Sum, VerbOp::Add, PAIR, false, false),
    row(Kw::Difference, VerbOp::Sub, PAIR, false, false),
    row(Kw::Product, VerbOp::Mul, PAIR, false, false),
    row(Kw::Divide, VerbOp::Div, DIVIDE, false, false),
    row(Kw::Remainder, VerbOp::Rem, DIVIDE, false, false),
    row(Kw::BitAnd, VerbOp::BitAnd, PAIR, false, false),
    row(Kw::BitOr, VerbOp::BitOr, PAIR, false, false),
    row(Kw::BitXor, VerbOp::BitXor, PAIR, false, false),
    row(Kw::BitNot, VerbOp::BitNot, &[Slot::Ei], false, false),
    row(Kw::Shift, VerbOp::Shift, &[Slot::Ko, Slot::Bit(Dir::Left)], false, false),
    row(Kw::Equal, VerbOp::Eq, EQUAL, false, true),
    row(Kw::NotEqual, VerbOp::Ne, EQUAL, false, true),
    row(Kw::Greater, VerbOp::Gt, GREATER, false, true),
    row(Kw::NotGreater, VerbOp::Le, GREATER, false, true),
    row(Kw::Less, VerbOp::Lt, LESS, false, true),
    row(Kw::NotLess, VerbOp::Ge, LESS, false, true),
    row(Kw::Convert, VerbOp::Cast, &[Slot::Ko, Slot::KindAhpjit], false, false),
    row(Kw::Print, VerbOp::Print, &[Slot::KoOrHu], true, false),
    row(Kw::Assign, VerbOp::Assign, &[Slot::Ko, Slot::Hu], true, false),
    row(Kw::ReturnValue, VerbOp::Return, &[Slot::Ko], true, false),
    row(Kw::Address, VerbOp::AddrOf, &[Slot::Ei], false, false),
    row(Kw::Value, VerbOp::Deref, &[Slot::Ei], false, false),
    row(Kw::Cell, VerbOp::Cell, &[Slot::Ei, Slot::Ordinal], false, false),
    row(Kw::CellCount, VerbOp::CellCount, &[Slot::Ei], false, false),
];

/// Find the valence table row of a verb token.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The row, or `None` when `kw` is no verb.
pub fn verb_for(kw: Kw) -> Option<&'static Verb> { VERBS.iter().find(|v| v.kw == kw) }
