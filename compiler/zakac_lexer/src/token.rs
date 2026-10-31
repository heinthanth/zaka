use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::AsRefStr;
use strum::EnumString;
use strum::IntoStaticStr;
use zakac_diagnostics::Diagnostic;
use zakac_span::span::Span;

/// A lexical token. Whitespace never becomes a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Token
{
    /// What the token is.
    pub kind: TokenKind,
    /// Source region of the token, including an absorbed particle.
    pub span: Span,
    /// Sentence-final particle absorbed into a predicate or boolean literal.
    #[builder(default)]
    pub particle: Particle,
    /// `true` when no whitespace separates this token from the previous one.
    pub joined: bool,
}

/// Kind of a [`Token`]; its static name is the snake case variant name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TokenKind
{
    /// Identifier; the text comes from the span.
    Ident,
    /// Identifier in backticks; the span covers the backticks.
    RawIdent,
    /// Number literal; the digits come from the span, the parser checks them.
    Number(NumberShape),
    /// Character literal with its decoded value.
    Char(char),
    /// String literal; index into [`LexOutput::strings`].
    Str(u32),
    /// Reserved word or joined reserved phrase.
    Kw(Kw),
    /// Punctuation or operator.
    Punct(Punct),
    /// End of input.
    Eof,
}

/// Sentence-final particle attached to a predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Particle
{
    /// No particle.
    #[default]
    None,
    /// `သည်`
    Thi,
    /// `ပါ`
    Pa,
    /// `ပါသည်`
    PaThi,
}

impl Particle
{
    /// Combine this particle with a following particle word.
    ///
    /// # Arguments
    ///
    /// * `kw` - The following reserved word.
    ///
    /// # Returns
    ///
    /// The combined particle, or `None` when `kw` cannot extend this one.
    pub fn with(self, kw: Kw) -> Option<Particle>
    {
        match (self, kw)
        {
            (Particle::None, Kw::Thi) => Some(Particle::Thi),
            (Particle::None, Kw::Pa) => Some(Particle::Pa),
            (Particle::Pa, Kw::Thi) => Some(Particle::PaThi),
            _ => None,
        }
    }
}

/// Script of the digits in a number literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigitScript
{
    /// `၀`–`၉`
    Burmese,
    /// `0`–`9`
    Western,
}

/// Shape of a number literal. The value is computed in the parser, which knows
/// the base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct NumberShape
{
    /// Script of the first digit.
    pub script: DigitScript,
    /// `digits . digits`
    pub has_fraction: bool,
    /// `digits [eE] [+-]? digits`; note that `1e5` is also a valid base-16
    /// literal, so the parser re-reads the digits after a base prefix.
    pub has_exponent: bool,
    /// ASCII letters other than an exponent marker; valid after a base prefix
    /// with base above 10.
    pub has_letters: bool,
}

/// Punctuation and operator tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum Punct
{
    /// `၊`
    #[strum(serialize = "၊")]
    ListSep,
    /// `။`
    #[strum(serialize = "။")]
    FullStop,
    /// `၏`
    #[strum(serialize = "၏")]
    Ei,
    /// `(`
    #[strum(serialize = "(")]
    LParen,
    /// `)`
    #[strum(serialize = ")")]
    RParen,
    /// `[`
    #[strum(serialize = "[")]
    LBracket,
    /// `]`
    #[strum(serialize = "]")]
    RBracket,
    /// `:`
    #[strum(serialize = ":")]
    Colon,
    /// `+`
    #[strum(serialize = "+")]
    Plus,
    /// `-`
    #[strum(serialize = "-")]
    Minus,
    /// `*`
    #[strum(serialize = "*")]
    Star,
    /// `/`
    #[strum(serialize = "/")]
    Slash,
    /// `%`
    #[strum(serialize = "%")]
    Percent,
    /// `&`
    #[strum(serialize = "&")]
    Amp,
    /// `|`
    #[strum(serialize = "|")]
    Pipe,
    /// `^`
    #[strum(serialize = "^")]
    Caret,
    /// `~`
    #[strum(serialize = "~")]
    Tilde,
    /// `!`
    #[strum(serialize = "!")]
    Bang,
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
    /// `==`
    #[strum(serialize = "==")]
    EqEq,
    /// `!=`
    #[strum(serialize = "!=")]
    Ne,
    /// `<<`
    #[strum(serialize = "<<")]
    Shl,
    /// `>>`
    #[strum(serialize = ">>")]
    Shr,
    /// `&&`
    #[strum(serialize = "&&")]
    AndAnd,
    /// `||`
    #[strum(serialize = "||")]
    OrOr,
}

/// Reserved words and joined reserved phrases.
///
/// Variants named `*Head` / `*Tail` and a few others are pieces of multi-word
/// phrases; the phrase joiner replaces them with one joined variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kw
{
    // particles
    /// `သည်`
    Thi,
    /// `ကို`
    Ko,
    /// `နှင့်`
    Hnint,
    /// `ဖြင့်`
    Hpjint,
    /// `ထက်`
    Htet,
    /// `အောက်`
    Auk,
    /// `ဟု`
    Hu,
    /// `အဖြစ်`
    As,
    /// `ပါ`
    Pa,
    /// `တို့ကို`
    ListKo,
    /// `တို့ဖြင့်`
    ListWith,
    /// `လက်ခံပြီး`
    Accepting,
    /// `မှ`
    From,
    /// `အထိ`
    To,
    /// `စီ`
    Step,
    /// `အနေဖြင့်`
    CountAs,
    /// `ဖြစ်ပြီး`, `ဖြစ်ကာ`
    Being,
    /// `ပြီးလျှင်`
    WhenDone,
    /// `ဘယ်သို့`
    ShiftLeft,
    /// `ညာသို့`
    ShiftRight,
    /// `ဘစ်`
    Bit,

    // verb-nouns and verbs
    /// `ပေါင်းလဒ်`
    Sum,
    /// `နုတ်လဒ်`
    Difference,
    /// `မြှောက်လဒ်`
    Product,
    /// `ညီ`
    Equal,
    /// `မညီ`
    NotEqual,
    /// `ကြီး`
    Greater,
    /// `မကြီး`
    NotGreater,
    /// `ငယ်`
    Less,
    /// `မငယ်`
    NotLess,
    /// `မဟုတ်`
    Not,
    /// `ဖြစ်၍`
    And,
    /// `သို့မဟုတ်`
    Or,

    // phrase pieces
    /// `စားသော`
    DivideHead,
    /// `စားလဒ်`
    QuotientTail,
    /// `စားကြွင်း`
    RemainderTail,
    /// `ဘစ်အလိုက်`
    BitwiseHead,
    /// `ဆုံလဒ်`
    MeetTail,
    /// `ပေါင်းစုလဒ်`
    UnionTail,
    /// `ခြားနားလဒ်`
    DifferTail,
    /// `ပြောင်းပြန်ပြောင်းလဒ်`
    InvertTail,
    /// `ရွေ့သော`
    ShiftHead,
    /// `ပြောင်းလဲသော`
    ConvertHead,
    /// `လုပ်ဆောင်သော`
    CallHead,
    /// `အောက်ပါ`
    Following,
    /// `လုပ်ဆောင်မည်`
    WillDo,
    /// `လုပ်ဆောင်ကာ`
    DoAnd,
    /// `လုပ်ဆောင်စရာများ`
    Tasks,
    /// `ပြီး(ဆုံး)?ပါပြီ(း)?`
    Done,

    // joined phrases
    /// `စားသော စားလဒ်`
    Divide,
    /// `စားသော စားကြွင်း`
    Remainder,
    /// `ဘစ်အလိုက် ဆုံလဒ်`
    BitAnd,
    /// `ဘစ်အလိုက် ပေါင်းစုလဒ်`
    BitOr,
    /// `ဘစ်အလိုက် ခြားနားလဒ်`
    BitXor,
    /// `ဘစ်အလိုက် ပြောင်းပြန်ပြောင်းလဒ်`
    BitNot,
    /// `ရွေ့သော ရလဒ်`
    Shift,
    /// `ပြောင်းလဲသော ရလဒ်`
    Convert,
    /// `လုပ်ဆောင်သော ရလဒ်`
    CallResult,
    /// `ရလဒ် အဖြစ် ပြန်ပေးပါ`
    ReturnValue,
    /// `အောက်ပါ တို့ကို လုပ်ဆောင်ပါ`
    BlockOpen,
    /// `အောက်ပါ တို့ကို လုပ်ဆောင်မည်`
    FnOpen,
    /// `နေရာယူ၍ လိပ်စာ ကို`
    AllocTarget,
    /// `မင်မိုရီ တွင်`; an allocation follows.
    AllocHead,
    /// `ရှိ မင်မိုရီ ကို`; a resize or a release follows.
    HeapHead,
    /// `အောက်ပါ တို့ကို လုပ်ဆောင်ကာ`; a result clause follows.
    FnOpenRet,
    /// `လုပ်ဆောင်စရာများ ပြီး(ဆုံး)?ပါပြီ(း)?`
    BlockClose,

    // statement verbs
    /// `ပြသပါ`
    Print,
    /// `ထားပါ`
    Assign,
    /// `ပြန်ပေးပါ`
    Return,
    /// `ပြန်ပေးမည်`
    WillReturn,
    /// `လုပ်ဆောင်ပါ`
    Call,
    /// `ဖြစ်ပါစေ`
    Let,
    /// `ရပ်ပါ`
    Stop,
    /// `ကျော်ပါ`
    Skip,

    // control
    /// `အကယ်၍`
    If,
    /// `လျှင်`
    Then,
    /// `ဖြစ်လျှင်`
    ThenIs,
    /// `ထိုသို့မဟုတ်ဘဲ`
    ElseIf,
    /// `သို့မဟုတ်ပါက`
    Else,
    /// `နေသရွေ့`
    While,
    /// `ဖြစ်နေသရွေ့`
    WhileIs,
    /// `ရေတွက်၍`
    Counting,
    /// `တိုး၍`
    Increasing,
    /// `လျှော့၍`
    Decreasing,
    /// `ကွင်းဆက်`
    Loop,
    /// `နောက်တစ်ကြိမ်ဆီသို့`
    NextIteration,

    // nouns
    /// `ကိန်းပြည့်`
    IntKind,
    /// `ဒဿမကိန်း`
    FloatKind,
    /// `စာလုံး`
    CharKind,
    /// `မှန်မှားတန်ဖိုး`
    BoolKind,
    /// `ဘူလီယန်`
    BoolKindLoan,
    /// `လက္ခဏာရှိ`
    Signed,
    /// `လက္ခဏာမဲ့`
    Unsigned,
    /// `အခြေ`
    Base,
    /// `ကိန်းသေ`
    Const,
    /// `ကိန်းရှင်`
    Var,
    /// `တန်ဖိုး`
    Value,
    /// `လုပ်ဆောင်ချက်`
    Function,
    /// `ရလဒ်`
    Result,
    /// `မှန်`
    True,
    /// `မမှန်`
    False,
    /// `အနုတ်`
    Negate,
    /// `မှတ်ချက်`, when it is part of a longer word made of reserved words.
    Note,

    // pointers and the heap
    /// `လိပ်စာ`
    Address,
    /// `လိပ်စာအကြမ်း`
    RawAddress,
    /// `ငှားထားသော`
    Borrowed,
    /// `တန်ဖိုးအငှား`
    Lend,
    /// `ဗလာ`
    Null,
    /// `အရွယ်အစား`
    SizeOf,
    /// `မင်မိုရီ` or `မှတ်ဉာဏ်`
    Mem,
    /// `ဘိုက်စာ`
    Octets,
    /// `ခုစာ`
    Items,
    /// `နေရာယူ၍`
    TakeSpace,
    /// `တွင်`
    At,
    /// `သိမ်းထားမည်`
    WillKeep,
    /// `ရှိ`
    Located,
    /// `ဖြစ်အောင်`
    SoThat,
    /// `နေရာအချဲ့အချုံ့လုပ်မည်`
    WillResize,
    /// `ပြန်လည်စွန့်လွှတ်မည်`
    WillRelease,

    // lists
    /// `စာရင်း`
    List,
    /// `စာရင်းအလွတ်`
    BlankList,
    /// `အကွက်`
    Cell,
    /// `အကွက်အရေအတွက်`
    CellCount,
    /// `ခုမြောက်`
    Ordinal,
    /// `ခုရှိသော` or `ရှိသော`
    Having,
    /// `တို့ပါသော`, before `စာရင်း` in a list literal
    ListOf,

    // structs
    /// `အချက်အလက်အမျိုးအစား` or `ဒေတာအမျိုးအစား`
    StructOpen,
    /// `တို့`, a piece of the field list closers
    Plural,
    /// `ပါဝင်သည်`
    Contains,
    /// `ပါဝင်ပြီး`
    ContainsAnd,
    /// `မျိုးပြပစ္စည်းမှာ`
    ClassifierIs,
    /// `ဖြစ်သည်`
    Is,
    /// `တို့ ပါဝင်သည်`
    FieldsClose,
    /// `တို့ ပါဝင်ပြီး`; a classifier clause follows.
    FieldsCloseAnd,
}

impl Kw
{
    /// Whether this word is a particle (a role marker).
    ///
    /// A word made of particles alone is not split into reserved words.
    ///
    /// # Returns
    ///
    /// `true` for particles.
    pub const fn is_particle(self) -> bool
    {
        matches!(
            self,
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
                | Kw::Accepting
                | Kw::From
                | Kw::To
                | Kw::Step
                | Kw::CountAs
                | Kw::Being
                | Kw::WhenDone
                | Kw::ShiftLeft
                | Kw::ShiftRight
                | Kw::Bit
                | Kw::ListOf
        )
    }

    /// Whether this word takes a sentence-final particle written attached to
    /// it: the comparison verbs, `မဟုတ်` and the boolean literals.
    ///
    /// Verb-nouns do not: in `ပေါင်းလဒ်သည်` the `သည်` is a subject marker.
    ///
    /// # Returns
    ///
    /// `true` for words that absorb a following `သည်` / `ပါ`.
    pub const fn absorbs_particle(self) -> bool
    {
        matches!(
            self,
            Kw::Equal
                | Kw::NotEqual
                | Kw::Greater
                | Kw::NotGreater
                | Kw::Less
                | Kw::NotLess
                | Kw::Not
                | Kw::True
                | Kw::False
        )
    }
}

/// Kind of a comment marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CommentKind
{
    /// `//`
    Slash,
    /// `///`
    DocSlash,
    /// `မှတ်ချက်`
    Note,
}

/// A comment, from the marker to the end of the line. The text comes from the
/// span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Comment
{
    /// The marker used.
    pub kind: CommentKind,
    /// Source region, from the marker to the end of the line (without the line
    /// break).
    pub span: Span,
}

/// A piece of a decoded string or character literal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrPart
{
    /// Literal text, with `{{` and `}}` folded.
    Text(String),
    /// A decoded escape sequence.
    Escape(char),
    /// Content of an unknown `{...}`, kept for future variable substitution.
    Hole(String),
}

/// A decoded string literal.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Builder)]
pub struct StrLit
{
    /// The decoded pieces in source order.
    pub parts: Vec<StrPart>,
    /// The pieces joined; a hole is kept as `{content}`.
    pub value: String,
}

/// The whole lexer output for one source file.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Builder)]
pub struct LexOutput
{
    /// Tokens in source order, ending with [`TokenKind::Eof`].
    #[builder(default)]
    pub tokens: Vec<Token>,
    /// Comments, in source order.
    #[builder(default)]
    pub comments: Vec<Comment>,
    /// Decoded string literals, indexed with [`TokenKind::Str`].
    #[builder(default)]
    pub strings: Vec<StrLit>,
    /// Lexer diagnostics, in source order.
    #[builder(default)]
    pub diagnostics: Vec<Diagnostic>,
}
