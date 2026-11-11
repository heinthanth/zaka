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
    /// `?`, after a call: pass a failure on
    #[strum(serialize = "?")]
    Question,
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
    /// `ထဲသို့`, before the value an add statement puts in a list
    Into,
    /// `မှ`
    From,
    /// `အထိ`
    To,
    /// `စီ`
    Step,
    /// `အနေဖြင့်`
    CountAs,
    /// `အပိုင်အနေဖြင့်`, before `ပြန်ပေးမည်` in the result clause of a C
    /// function whose text the program takes over
    AsOwned,
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
    /// `တစ်ခုတည်း`, "one and the same": whether two names hold one object
    SameOne,
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
    /// `အကွက်တိုင်း`, "each cell"
    EachCell,
    /// `စာလုံးတိုင်း`, "each character"
    EachChar,
    /// `ယူ၍`, "taking": opens the block of an each-cell loop
    Taking,
    /// `ပြောင်းလဲရန်`, "in order to change": the second word of
    /// `ဒေတာ ပြောင်းလဲရန်`
    ToChange,
    /// `ထည့်ပေးရန်`, "in order to put in": the second word of `ဒေတာ ထည့်ပေးရန်`
    ToFill,
    /// `ဒေတာ ပြောင်းလဲရန်`, "to change the data": a parameter the function can
    /// read and write, whose changes reach the caller's place, which the caller
    /// sets first
    DataToChange,
    /// `ဒေတာ ထည့်ပေးရန်`, "to put the data in": a parameter the function writes
    /// and never reads first, which the caller need not set; it counts as set
    /// after the call
    DataToFill,
    /// `ဝိဘတ်`, "particle": opens the mark of a parameter, `ဝိဘတ် "<mark>" ဖြင့်`
    MarkWord,
    /// `ဟူသော`, "called": before `လုပ်ဆောင်ချက်` in a function value
    Called,
    /// `လုပ်ဆောင်သော`
    CallHead,
    /// `အောက်ပါ`
    Following,
    /// `လုပ်ဆောင်မည်`
    WillDo,
    /// `မည်`, the end of the short call statement `<fn> မည်`
    Will,
    /// `သည့်`, the first piece of `သည့် ရလဒ်`
    Which,
    /// `သုံး၍`, "using": the arguments of a call written before the function
    Using,
    /// `ပြင်ပ`, "outside": the first piece of `ပြင်ပမှလာသော`
    Outside,
    /// `လာသော`, "coming": the last piece of `ပြင်ပမှလာသော`
    Coming,
    /// `အခြားတန်ဖိုးများ`, "other values": the last item of the parameters of a
    /// C function that takes more arguments than it names
    OtherValues,
    /// `နှင့်အတူ`, "together with": the word after `အခြားတန်ဖိုးများ` that
    /// ends the item form of the parameters of such a C function
    Along,
    /// `အခြားတန်ဖိုးများနှင့်အတူ`, "together with other values": ends the
    /// item form of the parameters of a C function that takes more
    /// arguments than it names
    WithOtherValues,
    /// `လုပ်ဆောင်ချက်တစ်ခု`, "a function", in the header of an external function
    OneFunction,
    /// `လက်ခံမည်`, "will accept": ends the parameters of an external function
    /// with no result
    WillAccept,
    /// `လက်ခံသော`, "that accepts": ends the parameters of a function kind
    /// with no result
    AcceptingRel,
    /// `ပြန်ပေးသော`, "that gives back": ends the result of a function kind
    ReturningRel,
    /// `လုပ်ဆောင်ချက် အမျိုးအစား`, "function kind": closes a function kind
    FnKind,
    /// `ဟူသော လုပ်ဆောင်ချက်`, "the function called": after a name, the
    /// function of that name as a value
    FnValue,
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
    /// `လုပ်ဆောင်သော ရလဒ်ကောင်း`: a call that passes a failure on
    CallGoodResult,
    /// `သည့် ရလဒ်`: the end of the short call expression `<fn> သည့် ရလဒ်`
    UsedResult,
    /// `ပြင်ပ မှ လာသော`, "coming from outside": marks a function the host
    /// serves
    External,
    /// `သည့် ရလဒ်ကောင်း`: the short call expression that passes a failure on
    UsedGoodResult,
    /// `မအောင်မြင်ပါက အမှားအယွင်း ကို ပြန်ပေးမည်`: a call statement passes a
    /// failure on
    PassFault,
    /// `လုပ်ဆောင်ချက် အောင်မြင်သည် ဟု သတ်မှတ်မည်`: the function ends with a
    /// good result that holds no value
    MarkSucceeded,
    /// `လုပ်ဆောင်ချက် မအောင်မြင်ပါ ဟု သတ်မှတ်မည်`: the function ends with a
    /// failure that holds no error value
    MarkFailed,
    /// `အောက်ပါ တို့ကို လုပ်ဆောင်မည်`: opens a block, or the steps of a function
    BlockOpen,
    /// `အောက်ပါ တို့ကို စိတ်မချရသော်လည်း လုပ်ဆောင်မည်`: opens a block of marked
    /// code
    UnsafeBlockOpen,
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
    /// `ပရိုဂရမ် ပြီး(ဆုံး)?ပါပြီ(း)?`: closes the start block.
    ProgramClose,

    // statement verbs
    /// `ပြသမည်`
    Print,
    /// `ထားမည်`
    Assign,
    /// `ပြန်ပေးမည်`
    WillReturn,
    /// `ဖြစ်ပါစေ`
    Let,
    /// `ရပ်မည်`
    Stop,
    /// `ကျော်မည်`
    Skip,
    /// `ထည့်သွင်းမည်`, which ends an import sentence
    Import,
    /// `ထည့်မည်`, which adds a cell to a list
    Add,
    /// `ဖျက်မည်`, which removes the last cell of a list
    Remove,
    /// `အစားထိုးမည်`, which replaces a range of text with a value
    Replace,

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
    /// `စာသား`
    TextKind,
    /// `စာလုံးအရေအတွက်`
    CharCount,
    /// `ဆက်ထားသည့်`, in `a၊ b တို့ကို ဆက်ထားသည့် စာသား`
    Joined,
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
    /// `ပရိုဂရမ်`: opens the start block, as in `ပရိုဂရမ်သည်`
    Program,
    /// `ရလဒ်`
    Result,
    /// `ရလဒ်ကောင်း`, the good result of a call that can fail
    GoodResult,
    /// `မအောင်မြင်ပါက`, "if it fails"
    OnFail,
    /// `အမှားအယွင်း`, an error value
    Fault,
    /// `အောင်မြင်သည့်သတင်း`, "news of success": the good side of a result
    /// kind with no good value
    SuccessNews,
    /// `မအောင်မြင်သည့်သတင်း`, "news of failure": the error side of a result
    /// kind with no error value
    FailureNews,
    /// `ရှိကောင်းရှိနိုင်သော`, "that might contain": an optional kind,
    /// `T ရှိကောင်းရှိနိုင်သော ဒေတာဆဲလ်`
    Perhaps,
    /// `ရှိနိုင်သော`, "that can hold": a result kind,
    /// `T သို့မဟုတ် အမှားအယွင်းအနေဖြင့် E ရှိနိုင်သော ဒေတာဆဲလ်`
    CanHold,
    /// `ဒေတာဆဲလ်`, "a data cell": ends an optional or a result kind
    DataCell,
    /// `အလွတ်`, "blank": the optional that holds no data
    NoData,
    /// `ကနဦးတန်ဖိုး`, "the initial value": the written default of the kind
    /// its place takes, and the mark of the default member of an enumeration
    Initial,
    /// `ဒေတာ` or `အချက်အလက်`, "data": what an optional holds
    Data,
    /// `ထဲတွင်`, "in": after the optional or the result a test looks in
    Within,
    /// `မရှိ` or `မရှိနေ`, "is not there"
    NotLocated,
    /// `ဒေတာ ရှိ`: a test that an optional holds data
    HasData,
    /// `ဒေတာ မရှိ`: a test that an optional holds no data
    LacksData,
    /// `ရလဒ်ကောင်း ရှိ` or `အောင်မြင်သည့်သတင်း ရှိ`: a test that a result is good
    HasGood,
    /// `အမှားအယွင်း ရှိ` or `မအောင်မြင်သည့်သတင်း ရှိ`: a test that a result failed
    HasFault,
    /// `အမှားအယွင်း ဒေတာ`, after `ထဲမှ`: the error value of a result
    FaultData,
    /// `ဟူသော အမှားအယွင်း`: an error value, `<value> ဟူသော အမှားအယွင်း`
    CalledFault,
    /// `အမှားအယွင်းအနေဖြင့်`: marks the error side of a result kind
    AsFault,
    /// `အောင်မြင်သည်`, "it succeeded"
    Succeeded,
    /// `မအောင်မြင်ပါ`, "it did not succeed"
    NotSucceeded,
    /// `သတ်မှတ်မည်`, "will mark"
    WillMark,
    /// `မှန်`
    True,
    /// `မမှန်`
    False,
    /// `အနုတ်`
    Negate,
    /// `မှတ်ချက်`, when it is part of a longer word made of reserved words.
    Note,
    /// `အများသုံး`, which marks a top-level declaration as exported
    Export,
    /// `မျှဝေနိုင်သော`, which marks a struct whose values are shared objects
    Shareable,
    /// `စိတ်မချရသော`, "not to be trusted": marks a function whose statements
    /// can work with pointers, and which marked code alone can call
    Unsafe,
    /// `စိတ်မချရသော်လည်း`, "though not to be trusted": in the phrase that
    /// opens a block of marked code
    UnsafeThough,

    // pointers and the heap
    /// `လိပ်စာ`
    Address,
    /// `မည်သည်ကိုမှ မညွှန်းသော လိပ်စာ`, "an address that points at nothing":
    /// the null pointer, joined from its three words
    Null,
    /// `မည်သည်ကိုမှ`, "nothing at all": the first word of the null pointer
    Nothing,
    /// `မညွှန်းသော`, "that points at no": the second word of the null pointer
    PointsAtNo,
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
    /// `ခေါ်ဆိုခွင့်`, "leave to call": the first piece of the statements that
    /// let C keep a function value and take that leave back
    CallLeave,
    /// `ပြုမည်`, "will grant": the last piece of `ခေါ်ဆိုခွင့် ပြုမည်`
    WillGrant,
    /// `ပြန်လည်ရုပ်သိမ်းမည်`, "will take back": the last piece of
    /// `ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်`
    WillRevoke,
    /// `ခေါ်ဆိုခွင့် ပြုမည်`: lets C keep a function value and call it after the
    /// call that gave it
    AllowCall,
    /// `ခေါ်ဆိုခွင့် ပြန်လည်ရုပ်သိမ်းမည်`: takes back the leave
    /// `ခေါ်ဆိုခွင့် ပြုမည်` gave
    RevokeCall,
    /// `ဆီသို့`
    Toward,
    /// `ကူးယူမည်`: a blit whose two ranges must not overlap
    WillBlit,
    /// `ပြောင်းထည့်မည်`: a blit whose two ranges can overlap, or a move of a
    /// value into a variable
    WillShift,
    /// `ကူးထည့်မည်`: a duplicate of a value into a variable; a shallow duplicate
    /// of an object
    WillDupIn,
    /// `အကုန်ကူးထည့်မည်`: a deep duplicate of a value into a variable: each
    /// object inside is copied too
    WillDupAllIn,
    /// `အကုန်`: the word before `ကူးထည့်မည်` in a deep duplicate, written apart
    All,
    /// `ဖြည့်မည်`
    WillFill,

    // lists
    /// `စာရင်း`
    List,
    /// `စာရင်းအလွတ်`
    BlankList,
    /// `အကွက်`
    Cell,
    /// `အကွက်အရေအတွက်`
    CellCount,
    /// `နောက်ဆုံးအကွက်`, the last cell in a remove statement
    LastCell,
    /// `နောက်ဆုံးစာလုံး`, the last character in a remove statement
    LastChar,
    /// `အရှေ့တွင်`, after `<cell> ၏` or `<character> ၏` in an add statement:
    /// the new value goes in front of it
    Before,
    /// `ခုမြောက်` or `လုံးမြောက်`, after the number of a cell or a character
    /// counted from 1
    Ordinal,
    /// `ခုရှိသော` or `ရှိသော`
    Having,
    /// `တို့ပါသော`, before `စာရင်း` in a list literal
    ListOf,
    /// `မှစ၍`, between the start and the length of a range
    StartingAt,
    /// `ထိ`, after the last cell or character of the range a replace
    /// statement changes; no particle, so `ထိကို` splits into two words
    Upto,
    /// `ကွက်`, after the length of a range
    CellWord,

    // structs
    /// `အချက်အလက်အမျိုးအစား`, also written `ဒေတာအမျိုးအစား`: opens a kind alias
    DataKindOpen,
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

    // enumerations and switch
    /// `အမျိုးအစား`, "kind": a piece of the function kind phrase
    KindWord,
    /// `ထဲမှ`, between `<name>များ` and a member: `အရောင်များထဲမှ နီ`
    Among,
    /// `တခြား`, the opener of the default arm of a switch
    Other,

    // kind aliases
    /// `လည်း`, "also": a piece of the alias closer
    Also,
    /// `လည်းကောင်း`, "as well as": ends each item of a list of parameters
    /// that takes captures
    AsWellAs,
    /// `မှတ်ယူနိုင်ပါသည်`, "can be taken as": a piece of the alias closer
    CanRegard,
    /// `ဟုလည်း မှတ်ယူနိုင်ပါသည်`, "can also be taken as": closes a kind alias,
    /// `ဒေတာအမျိုးအစား <alias> ကို <kind> ဟုလည်း မှတ်ယူနိုင်ပါသည်။`
    AliasClose,
    /// `မှတ်ယူနိုင်ပြီး`, "can be taken as, and": a piece of the alias closer
    /// that a classifier clause follows
    CanRegardAnd,
    /// `ဟုလည်း မှတ်ယူနိုင်ပြီး`: closes a kind alias; a classifier clause follows
    AliasCloseAnd,

    // the declarations of structs and enumerations
    /// `အချက်အလက်အစုအဝေး`, also written `ဒေတာအစုအဝေး`: the word of a struct
    /// declaration, `<name> ဟူသော ဒေတာအစုအဝေးတွင် ...`
    StructWord,
    /// `အချက်အလက်အမျိုးအစားကွဲ`, also written `ဒေတာအမျိုးအစားကွဲ`: the word of an
    /// enumeration declaration, `<name> ဟူသော ဒေတာအမျိုးအစားကွဲတွင် ...`
    EnumWord,
    /// `သုံးမည့်`, "to be used": a piece of the mark of the initial variant
    ToUse,
    /// `ကနဦးတန်ဖိုးအနေဖြင့် သုံးမည့်`: marks the variant that is the initial
    /// value of its enumeration
    InitialMark,
    /// `တန်ဖိုးအနေဖြင့်`, "as its value": opens the value of a variant
    ValueAs,
    /// `ရှိသည့်`, "which has": closes the value of a variant
    HavingRel,
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
                | Kw::Into
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
    /// A `{...}` in a string literal: the text of an expression whose value
    /// takes its place.
    TemplateSlot
    {
        /// The text between the braces.
        text: String,
        /// The source region of that text.
        span: Span,
    },
    /// An escape that does not decode, kept as written; the lexer reports
    /// it.
    Bad(String),
}

/// A decoded string literal.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Builder)]
pub struct StrLit
{
    /// The decoded pieces in source order.
    pub parts: Vec<StrPart>,
    /// The pieces joined; a template slot is kept as `{content}`.
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
