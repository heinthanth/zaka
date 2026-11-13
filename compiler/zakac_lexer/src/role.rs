use strum::IntoStaticStr;
use zakac_rust_sugars::tern;

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
    /// An identifier in backticks.
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
        Kw::Sum
        | Kw::Difference
        | Kw::Product
        | Kw::Equal
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
        | Kw::ShiftLeft
        | Kw::ShiftRight
        | Kw::DivideHead
        | Kw::QuotientTail
        | Kw::RemainderTail
        | Kw::BitwiseHead
        | Kw::MeetTail
        | Kw::UnionTail
        | Kw::DifferTail
        | Kw::InvertTail
        | Kw::ShiftHead
        | Kw::ConvertHead
        | Kw::CallHead
        | Kw::Divide
        | Kw::Remainder
        | Kw::BitAnd
        | Kw::BitOr
        | Kw::BitXor
        | Kw::BitNot
        | Kw::Shift
        | Kw::Convert
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
        | Kw::MarkWord
        | Kw::Called
        | Kw::Named
        | Kw::ListOf
        | Kw::ItemOf
        | Kw::StartingAt
        | Kw::Upto
        | Kw::Having
        | Kw::HavingRel => TokenRole::Particle,
        Kw::IntKind
        | Kw::FloatKind
        | Kw::CharKind
        | Kw::TextKind
        | Kw::BoolKind
        | Kw::BoolKindLoan
        | Kw::Signed
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
