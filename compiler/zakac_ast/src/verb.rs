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
    /// `ထဲသို့`, after the list an add statement grows.
    #[strum(serialize = "ထဲသို့")]
    Into,
    /// `၏ အရှေ့တွင်`, after the cell or the character an add statement puts
    /// the value in front of; it fills the role of `ထဲသို့`.
    #[strum(serialize = "၏ အရှေ့တွင်")]
    Before,
    /// `ထိကို`, after the range of text a replace statement changes.
    #[strum(serialize = "ထိကို")]
    Upto,
    /// `ထဲတွင်`, after the optional or the result a test looks in.
    #[strum(serialize = "ထဲတွင်")]
    Within,
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
        ret_true!(matches!((self, want), (Slot::Before, Slot::Into)));
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
    /// `တစ်ခုတည်း`: whether two names hold one object or one function value.
    #[strum(serialize = "same")]
    Same,
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
    /// `ပြသမည်`
    Print,
    /// `ထားမည်`
    #[strum(serialize = "set")]
    Assign,
    /// `<v> ကို ပြန်ပေးမည်`
    Return,
    /// `<r> ထဲတွင် ရလဒ်ကောင်း ရှိ`: whether a result holds a good result.
    #[strum(serialize = "is_good")]
    IsGood,
    /// `<r> ထဲတွင် အမှားအယွင်း ရှိ`: whether a result holds an error value.
    #[strum(serialize = "is_fault")]
    IsFault,
    /// `<x> ထဲတွင် ဒေတာ ရှိ`: whether an optional holds data.
    #[strum(serialize = "has_data")]
    HasData,
    /// `<x> ထဲတွင် ဒေတာ မရှိ`: whether an optional holds no data.
    #[strum(serialize = "lacks_data")]
    LacksData,
    /// `<value> ဟူသော အမှားအယွင်း`: an error value of the result kind the
    /// place wants.
    #[strum(serialize = "make_fault")]
    MakeFault,
    /// `<x> ထဲမှ ဒေတာ`: the data of an optional, or the good result of a
    /// result; an optional with no data, or a result that holds an error, stops
    /// the run. `<r> ကို ရလဒ်ကောင်း အဖြစ် ပြောင်းလဲသော ရလဒ်` is the old spelling.
    #[strum(serialize = "unwrap")]
    Unwrap,
    /// `<r> ကို အမှားအယွင်း အဖြစ် ပြောင်းလဲသော ရလဒ်`: the error value; a good
    /// result stops the run.
    #[strum(serialize = "unwrap_fault")]
    UnwrapFault,
    /// `ထည့်မည်`: a new last cell of a list.
    #[strum(serialize = "add_cell")]
    AddCell,
    /// `ကူးထည့်မည်`: a duplicate of a value goes into a variable.
    #[strum(serialize = "dup_into")]
    DupInto,
    /// `အကုန်ကူးထည့်မည်`: a deep duplicate of a value goes into a variable.
    #[strum(serialize = "deep_dup_into")]
    DeepDupInto,
    /// `၏ မိတ္တူ`: a duplicate of a value; a new object with the same fields
    /// for an object.
    #[strum(serialize = "dup_of")]
    DupOf,
    /// `တစ်<classifier>လုံး ၏ မိတ္တူ`: a deep duplicate, each object inside
    /// duplicated too, once each.
    #[strum(serialize = "deep_dup_of")]
    DeepDupOf,
    /// `အစားထိုးမည်`: a range of text gives its place to a value.
    #[strum(serialize = "replace")]
    Replace,
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
    /// `၏ စာလုံးအရေအတွက်`: the number of characters of text.
    #[strum(serialize = "char_count")]
    CharCount,
    /// `၏ စာလုံး i`: the character of text at a number, counted from 0.
    /// The parser builds it; no verb word stands for it.
    #[strum(serialize = "char_at")]
    CharAt,
    /// `၏ n ခုမြောက်စာလုံး`: the character of text at a number, counted
    /// from 1. The parser builds it; no verb word stands for it.
    #[strum(serialize = "char1")]
    CharNth,
    /// `a၊ b တို့ကို ဆက်ထားသည့် စာသား`: new text that joins each piece of
    /// text in order. The parser builds it from the list of pieces.
    #[strum(serialize = "join")]
    Join,
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
    row(Kw::SameOne, VerbOp::Same, &[Slot::Hnint, Slot::Thi], false, true),
    row(Kw::Greater, VerbOp::Gt, GREATER, false, true),
    row(Kw::NotGreater, VerbOp::Le, GREATER, false, true),
    row(Kw::Less, VerbOp::Lt, LESS, false, true),
    row(Kw::NotLess, VerbOp::Ge, LESS, false, true),
    row(Kw::Convert, VerbOp::Cast, &[Slot::Ko, Slot::KindAhpjit], false, false),
    row(Kw::Print, VerbOp::Print, &[Slot::KoOrHu], true, false),
    row(Kw::Assign, VerbOp::Assign, &[Slot::Ko, Slot::Hu], true, false),
    row(Kw::WillReturn, VerbOp::Return, &[Slot::Ko], true, false),
    row(Kw::HasGood, VerbOp::IsGood, &[Slot::Within], false, false),
    row(Kw::HasFault, VerbOp::IsFault, &[Slot::Within], false, false),
    row(Kw::HasData, VerbOp::HasData, &[Slot::Within], false, false),
    row(Kw::LacksData, VerbOp::LacksData, &[Slot::Within], false, false),
    row(Kw::Add, VerbOp::AddCell, &[Slot::Into, Slot::Ko], true, false),
    row(Kw::WillDupIn, VerbOp::DupInto, &[Slot::Into, Slot::Ko], true, false),
    row(Kw::WillDupAllIn, VerbOp::DeepDupInto, &[Slot::Into, Slot::Ko], true, false),
    row(Kw::Replace, VerbOp::Replace, &[Slot::Upto, Slot::Hpjint], true, false),
    row(Kw::Address, VerbOp::AddrOf, &[Slot::Ei], false, false),
    row(Kw::Value, VerbOp::Deref, &[Slot::Ei], false, false),
    row(Kw::Cell, VerbOp::Cell, &[Slot::Ei, Slot::Ordinal], false, false),
    row(Kw::CellCount, VerbOp::CellCount, &[Slot::Ei], false, false),
    row(Kw::CharCount, VerbOp::CharCount, &[Slot::Ei], false, false),
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
