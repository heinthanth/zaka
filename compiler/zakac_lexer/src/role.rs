use strum::IntoStaticStr;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;

use crate::reserved::is_kind_name;
use crate::token::Kw;
use crate::token::Punct;
use crate::token::TokenKind;

/// The role of a token in a sentence, which decides its color in the site,
/// the explorer and the editors. Its static name is the snake case variant
/// name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum TokenRole
{
    /// A control word: a branch, a loop, an opener, a closer or a declaration.
    Control,
    /// A statement verb.
    Verb,
    /// An operator, in words or in signs.
    Op,
    /// A particle that links the parts of a sentence.
    Particle,
    /// A kind noun.
    Kind,
    /// A truth value, the null address or the blank list.
    Truth,
    /// An identifier.
    Ident,
    /// An identifier in braces, `{name}`.
    Raw,
    /// A number literal.
    Number,
    /// A character literal.
    Char,
    /// A string literal.
    Str,
    /// A separator or a bracket.
    Delim,
    /// A comment.
    Comment,
    /// The end of the input.
    Plain,
}

/// The role of a token kind.
///
/// # Arguments
///
/// * `kind` - The token kind.
///
/// # Returns
///
/// The role.
pub fn token_role(kind: &TokenKind) -> TokenRole
{
    match kind
    {
        TokenKind::Ident => TokenRole::Ident,
        TokenKind::RawIdent => TokenRole::Raw,
        TokenKind::Number(_) => TokenRole::Number,
        TokenKind::Char(_) => TokenRole::Char,
        TokenKind::Str(_) => TokenRole::Str,
        TokenKind::Kw(kw) => kw_role(*kw),
        TokenKind::Punct(punct) => punct_role(*punct),
        TokenKind::Eof => TokenRole::Plain,
    }
}

/// The role of a token, read with its text: a kind name the compiler gives,
/// such as `ကိန်းပြည့်`, is an identifier painted as a kind.
///
/// # Arguments
///
/// * `kind` - The token kind.
/// * `text` - The token text.
///
/// # Returns
///
/// The role.
pub fn role_with_text(kind: &TokenKind, text: &str) -> TokenRole
{
    ret_if!(*kind == TokenKind::Ident && is_kind_name(text), TokenRole::Kind);
    ret_if!(*kind == TokenKind::Ident && is_operator_word(text), TokenRole::Op);
    ret_if!(*kind == TokenKind::Ident && is_builtin_verb_word(text), TokenRole::Verb);
    ret_if!(*kind == TokenKind::Ident && builtin_count(text).is_some(), TokenRole::Op);
    token_role(kind)
}

/// The functions of the prelude the compiler gives whose sentence ends in
/// `<name> မည်`, with the reserved word of the verb each stands for.
pub const BUILTIN_VERBS: &[(&str, Kw)] = &[
    ("ပြသ", Kw::Print),
    ("ထည့်", Kw::Add),
    ("ဖျက်", Kw::Remove),
    ("အစားထိုး", Kw::Replace),
];

/// The functions of the prelude the compiler gives that are read right after
/// `၏`, with the reserved word of the reading each stands for.
pub const BUILTIN_COUNTS: &[(&str, Kw)] = &[
    ("အကွက်အရေအတွက်", Kw::CellCount),
    ("စာလုံးအရေအတွက်", Kw::CharCount),
    ("အညွှန်းအရေအတွက်", Kw::IndexCount),
];

/// The verb a function the compiler gives stands for.
///
/// # Arguments
///
/// * `name` - The name of the function.
///
/// # Returns
///
/// The reserved word of the verb, or `None`.
pub fn builtin_verb(name: &str) -> Option<Kw>
{
    BUILTIN_VERBS.iter().find(|(w, _)| *w == name).map(|(_, kw)| *kw)
}

/// The reading a count the compiler gives stands for.
///
/// # Arguments
///
/// * `name` - The name of the count.
///
/// # Returns
///
/// The reserved word of the reading, or `None`.
pub fn builtin_count(name: &str) -> Option<Kw>
{
    BUILTIN_COUNTS.iter().find(|(w, _)| *w == name).map(|(_, kw)| *kw)
}

/// Whether a word is the name of a verb the compiler gives with `မည်` glued
/// to it, `ပြသမည်`, so it is painted as a verb.
///
/// # Arguments
///
/// * `text` - The word.
///
/// # Returns
///
/// `true` for a glued verb.
fn is_builtin_verb_word(text: &str) -> bool
{
    text.strip_suffix("မည်").and_then(builtin_verb).is_some()
}

/// The role of a reserved word in a sentence: control word, statement verb,
/// operator, particle, kind noun or truth value.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The role.
pub fn kw_role(kw: Kw) -> TokenRole
{
    match kw
    {
        Kw::If
        | Kw::Then
        | Kw::ThenIs
        | Kw::ElseIf
        | Kw::Else
        | Kw::While
        | Kw::WhileIs
        | Kw::Counting
        | Kw::EachCell
        | Kw::EachChar
        | Kw::EachIndex
        | Kw::EachRow
        | Kw::Taking
        | Kw::Increasing
        | Kw::Decreasing
        | Kw::Loop
        | Kw::NextIteration
        | Kw::BlockOpen
        | Kw::FnOpenRet
        | Kw::BlockClose
        | Kw::ProgramClose
        | Kw::Following
        | Kw::WillDo
        | Kw::Will
        | Kw::Using
        | Kw::Outside
        | Kw::Coming
        | Kw::OtherValues
        | Kw::Along
        | Kw::WithOtherValues
        | Kw::External
        | Kw::OneFunction
        | Kw::WillAccept
        | Kw::AcceptingRel
        | Kw::ReturningRel
        | Kw::DoAnd
        | Kw::Tasks
        | Kw::Done
        | Kw::Accepting
        | Kw::OnFail
        | Kw::PassFault
        | Kw::Being
        | Kw::WhenDone
        | Kw::Function
        | Kw::Program
        | Kw::Const
        | Kw::Var
        | Kw::Mem
        | Kw::TakeSpace
        | Kw::At
        | Kw::Located
        | Kw::SoThat
        | Kw::AllocTarget
        | Kw::AllocHead
        | Kw::HeapHead
        | Kw::DataKindOpen
        | Kw::GenericKind
        | Kw::Substitutable
        | Kw::Itself
        | Kw::SubstitutedRel
        | Kw::Substituting
        | Kw::Trait
        | Kw::ToHave
        | Kw::MustContain
        | Kw::ToAddRel
        | Kw::Functions
        | Kw::Here
        | Kw::NoteThat
        | Kw::Notes
        | Kw::Has
        | Kw::InCompiler
        | Kw::CalledRel
        | Kw::IncludedWith
        | Kw::Plural
        | Kw::FieldsClose
        | Kw::FieldsCloseAnd
        | Kw::ClassifierIs
        | Kw::KindWord
        | Kw::Among
        | Kw::Other
        | Kw::AliasClose
        | Kw::Also
        | Kw::AsWellAs
        | Kw::CanRegard
        | Kw::CanRegardAnd
        | Kw::AliasCloseAnd
        | Kw::StructWord
        | Kw::EnumWord
        | Kw::ToUse
        | Kw::InitialMark
        | Kw::ValueAs
        | Kw::Export
        | Kw::Shareable
        | Kw::Unsafe
        | Kw::UnsafeThough
        | Kw::UnsafeBlockOpen => TokenRole::Control,
        Kw::Print
        | Kw::Assign
        | Kw::WillReturn
        | Kw::Let
        | Kw::Stop
        | Kw::Skip
        | Kw::MarkSucceeded
        | Kw::MarkFailed
        | Kw::WillMark
        | Kw::WillKeep
        | Kw::WillResize
        | Kw::WillRelease
        | Kw::CallLeave
        | Kw::WillGrant
        | Kw::WillRevoke
        | Kw::AllowCall
        | Kw::RevokeCall
        | Kw::WillBlit
        | Kw::WillShift
        | Kw::WillDupIn
        | Kw::WillDupAllIn
        | Kw::All
        | Kw::WillFill
        | Kw::Contains
        | Kw::ContainsAnd
        | Kw::Is
        | Kw::Add
        | Kw::Remove
        | Kw::Replace
        | Kw::Import => TokenRole::Verb,
        Kw::Equal
        | Kw::SameOne
        | Kw::NotEqual
        | Kw::Greater
        | Kw::NotGreater
        | Kw::Less
        | Kw::NotLess
        | Kw::Not
        | Kw::And
        | Kw::Or
        | Kw::Negate
        | Kw::ConvertHead
        | Kw::CallHead
        | Kw::Convert
        | Kw::ConvertGood
        | Kw::CallResult
        | Kw::CallGoodResult
        | Kw::Which
        | Kw::UsedResult
        | Kw::UsedGoodResult
        | Kw::SizeOf
        | Kw::Cell
        | Kw::CellCount
        | Kw::IndexWord
        | Kw::IndexCount
        | Kw::LastCell
        | Kw::LastChar
        | Kw::Before
        | Kw::CharCount
        | Kw::Joined
        | Kw::CellWord
        | Kw::Ordinal => TokenRole::Op,
        Kw::Thi
        | Kw::Ko
        | Kw::Hnint
        | Kw::Hpjint
        | Kw::Htet
        | Kw::Auk
        | Kw::Hu
        | Kw::As
        | Kw::Pa
        | Kw::ListKo
        | Kw::ListWith
        | Kw::Into
        | Kw::Toward
        | Kw::From
        | Kw::To
        | Kw::Step
        | Kw::CountAs
        | Kw::AsOwned
        | Kw::ToChange
        | Kw::ToFill
        | Kw::DataToChange
        | Kw::DataToFill
        | Kw::PrepositionWord
        | Kw::Called
        | Kw::Named
        | Kw::That
        | Kw::ListOf
        | Kw::ItemOf
        | Kw::StartingAt
        | Kw::Upto
        | Kw::Having
        | Kw::HavingRel => TokenRole::Particle,
        Kw::Signed
        | Kw::Unsigned
        | Kw::Bit
        | Kw::Base
        | Kw::Value
        | Kw::Result
        | Kw::GoodResult
        | Kw::Fault
        | Kw::SuccessNews
        | Kw::FailureNews
        | Kw::Perhaps
        | Kw::CanHold
        | Kw::DataCell
        | Kw::NoData
        | Kw::Data
        | Kw::Within
        | Kw::NotLocated
        | Kw::HasData
        | Kw::LacksData
        | Kw::HasGood
        | Kw::HasFault
        | Kw::FaultData
        | Kw::CalledFault
        | Kw::AsFault
        | Kw::Succeeded
        | Kw::NotSucceeded
        | Kw::Note
        | Kw::Address
        | Kw::FnKind
        | Kw::FnValue
        | Kw::Octets
        | Kw::Items
        | Kw::Table
        | Kw::Indexed
        | Kw::List => TokenRole::Kind,
        Kw::True
        | Kw::False
        | Kw::Null
        | Kw::Nothing
        | Kw::PointsAtNo
        | Kw::BlankList
        | Kw::BlankTable
        | Kw::Initial => TokenRole::Truth,
    }
}

/// The role of a punctuation token: operators stand out, separators and
/// brackets recede.
///
/// # Arguments
///
/// * `punct` - The punctuation.
///
/// # Returns
///
/// The role.
pub fn punct_role(punct: Punct) -> TokenRole
{
    let delim = matches!(
        punct,
        Punct::ListSep | Punct::FullStop | Punct::Ei | Punct::LParen | Punct::RParen
    );
    tern!(delim, TokenRole::Delim, TokenRole::Op)
}

/// The functions of the prelude that the word operators call, as
/// `က နှင့် ခ ကို ပေါင်းသည့်ရလဒ်` calls `ပေါင်း`.
pub const OPERATOR_NAMES: &[&str] = &[
    "ပေါင်း",
    "နုတ်",
    "မြှောက်",
    "စား",
    "အကြွင်းရှာ",
    "ဘစ်ဆုံ",
    "ဘစ်ပေါင်းစု",
    "ဘစ်ခြားနား",
    "ဘစ်ပြောင်းပြန်",
    "ဘစ်ဘယ်ရွေ့",
    "ဘစ်ညာရွေ့",
];

/// The value ends a word operator can have glued to it.
const VALUE_ENDS: &[&str] =
    &["သည့်ရလဒ်ကောင်း", "မည့်ရလဒ်ကောင်း", "သောရလဒ်ကောင်း", "သည့်ရလဒ်", "မည့်ရလဒ်", "သောရလဒ်"];

/// Whether a name is a word operator: the name of a function of
/// [`OPERATOR_NAMES`], alone or with a value end glued to it, so it is painted
/// as an operator.
///
/// # Arguments
///
/// * `text` - The name.
///
/// # Returns
///
/// `true` for a word operator.
pub fn is_operator_word(text: &str) -> bool
{
    let stem = VALUE_ENDS.iter().find_map(|end| text.strip_suffix(end));
    OPERATOR_NAMES.contains(&stem.unwrap_or(text))
}
