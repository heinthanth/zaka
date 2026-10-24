use std::collections::HashMap;
use std::sync::LazyLock;

use zakac_rust_sugars::ret_if;
use zakac_span::span::Span;

use crate::token::Kw;
use crate::words::Word;

/// Text of the comment marker word.
pub const NOTE_MARKER: &str = "မှတ်ချက်";

/// Reserved words, in normalized spelling. Several spellings can
/// map to one [`Kw`].
///
/// Multi-word phrases are not here: their pieces are, and
/// [`crate::phrases::join_phrases`] joins them.
#[rustfmt::skip]
pub static RESERVED: &[(&str, Kw)] = &[
    // particles
    ("သည်", Kw::Thi),
    ("ကို", Kw::Ko),
    ("နှင့်", Kw::Hnint),
    ("ဖြင့်", Kw::Hpjint),
    ("ထက်", Kw::Htet),
    ("အောက်", Kw::Auk),
    ("ဟု", Kw::Hu),
    ("အဖြစ်", Kw::As),
    ("ပါ", Kw::Pa),
    ("တို့ကို", Kw::ListKo),
    ("တို့ဖြင့်", Kw::ListWith),
    ("လက်ခံပြီး", Kw::Accepting),
    ("မှ", Kw::From),
    ("အထိ", Kw::To),
    ("စီ", Kw::Step),
    ("အနေဖြင့်", Kw::CountAs),
    ("ဖြစ်ပြီး", Kw::Being),
    ("ပြီးလျှင်", Kw::WhenDone),
    ("ဘယ်သို့", Kw::ShiftLeft),
    ("ညာသို့", Kw::ShiftRight),
    ("ဘစ်", Kw::Bit),
    // verb-nouns and verbs
    ("ပေါင်းလဒ်", Kw::Sum),
    ("နုတ်လဒ်", Kw::Difference),
    ("မြှောက်လဒ်", Kw::Product),
    ("ညီ", Kw::Equal),
    ("မညီ", Kw::NotEqual),
    ("ကြီး", Kw::Greater),
    ("မကြီး", Kw::NotGreater),
    ("ငယ်", Kw::Less),
    ("မငယ်", Kw::NotLess),
    ("မဟုတ်", Kw::Not),
    ("ဖြစ်၍", Kw::And),
    ("သို့မဟုတ်", Kw::Or),
    // phrase pieces
    ("စားသော", Kw::DivideHead),
    ("စားလဒ်", Kw::QuotientTail),
    ("စားကြွင်း", Kw::RemainderTail),
    ("ဘစ်အလိုက်", Kw::BitwiseHead),
    ("ဆုံလဒ်", Kw::MeetTail),
    ("ပေါင်းစုလဒ်", Kw::UnionTail),
    ("ခြားနားလဒ်", Kw::DifferTail),
    ("ပြောင်းပြန်ပြောင်းလဒ်", Kw::InvertTail),
    ("ရွေ့သော", Kw::ShiftHead),
    ("ပြောင်းလဲသော", Kw::ConvertHead),
    ("လုပ်ဆောင်သော", Kw::CallHead),
    ("အောက်ပါ", Kw::Following),
    ("လုပ်ဆောင်မည်", Kw::WillDo),
    ("လုပ်ဆောင်ကာ", Kw::DoAnd),
    ("လုပ်ဆောင်စရာများ", Kw::Tasks),
    ("ပြီးပါပြီ", Kw::Done),
    ("ပြီးပါပြီး", Kw::Done),
    ("ပြီးဆုံးပါပြီ", Kw::Done),
    ("ပြီးဆုံးပါပြီး", Kw::Done),
    // statement verbs
    ("ပြသပါ", Kw::Print),
    ("ထားပါ", Kw::Assign),
    ("ပြန်ပေးပါ", Kw::Return),
    ("ပြန်ပေးမည်", Kw::WillReturn),
    ("လုပ်ဆောင်ပါ", Kw::Call),
    ("ဖြစ်ပါစေ", Kw::Let),
    ("ရပ်ပါ", Kw::Stop),
    ("ကျော်ပါ", Kw::Skip),
    // control
    ("အကယ်၍", Kw::If),
    ("လျှင်", Kw::Then),
    ("ဖြစ်လျှင်", Kw::ThenIs),
    ("ထိုသို့မဟုတ်ဘဲ", Kw::ElseIf),
    ("သို့မဟုတ်ပါက", Kw::Else),
    ("နေသရွေ့", Kw::While),
    ("ဖြစ်နေသရွေ့", Kw::WhileIs),
    ("ရေတွက်၍", Kw::Counting),
    ("တိုး၍", Kw::Increasing),
    ("လျှော့၍", Kw::Decreasing),
    ("ကွင်းဆက်", Kw::Loop),
    ("နောက်တစ်ကြိမ်ဆီသို့", Kw::NextIteration),
    // nouns
    ("ကိန်းပြည့်", Kw::IntKind),
    ("ဒဿမကိန်း", Kw::FloatKind),
    ("စာလုံး", Kw::CharKind),
    ("မှန်မှားတန်ဖိုး", Kw::BoolKind),
    ("ဘူလီယန်", Kw::BoolKindLoan),
    ("လက္ခဏာရှိ", Kw::Signed),
    ("လက္ခဏာမဲ့", Kw::Unsigned),
    ("အခြေ", Kw::Base),
    ("ကိန်းသေ", Kw::Const),
    ("ကိန်းရှင်", Kw::Var),
    ("တန်ဖိုး", Kw::Value),
    ("လုပ်ဆောင်ချက်", Kw::Function),
    ("ရလဒ်", Kw::Result),
    ("မှန်", Kw::True),
    ("မမှန်", Kw::False),
    ("အနုတ်", Kw::Negate),
    (NOTE_MARKER, Kw::Note),
];

/// [`RESERVED`] as a map, built on first use.
static LOOKUP: LazyLock<HashMap<&'static str, Kw>> =
    LazyLock::new(|| RESERVED.iter().copied().collect());

/// Look up a whole word in the reserved table.
///
/// # Arguments
///
/// * `text` - The word, in normalized spelling.
///
/// # Returns
///
/// The reserved word, or `None`.
pub fn lookup(text: &str) -> Option<Kw> { LOOKUP.get(text).copied() }

/// A split of a word prefix into reserved words.
type Pieces = Vec<(Kw, Span)>;

/// Extend a prefix split with the piece `pre[start..end]`.
///
/// # Arguments
///
/// * `prefix` - The best split of `pre[..start]`, if one exists.
/// * `word` - The word being split.
/// * `start` - Index of the first pre-token of the piece.
/// * `end` - Index one past the last pre-token of the piece.
///
/// # Returns
///
/// The extended split, or `None` when there is no prefix split or the piece
/// is not reserved.
fn extend(
    prefix: &Option<Pieces>, word: &Word<'_>, start: usize, end: usize,
) -> Option<Pieces>
{
    let prefix = prefix.as_ref()?;
    let span = Span::new(word.pre[start].span.lo, word.pre[end - 1].span.hi);
    let lo = (span.lo - word.span.lo) as usize;
    let hi = (span.hi - word.span.lo) as usize;

    let kw = lookup(&word.text[lo..hi])?;
    let mut pieces = prefix.clone();
    pieces.push((kw, span));
    Some(pieces)
}

/// Split a word into reserved words.
///
/// Splits happen at pre-token (vanna) boundaries. The split with the fewest
/// pieces wins; a split made of particles alone is rejected, so `ကိုကို` remains
/// an identifier.
///
/// # Arguments
///
/// * `word` - The word to split.
///
/// # Returns
///
/// The pieces with their spans, or `None` when the word cannot be split into
/// two or more reserved words.
pub fn split_reserved(word: &Word<'_>) -> Option<Pieces>
{
    let n = word.pre.len();
    ret_if!(n < 2, None);

    let mut best: Vec<Option<Pieces>> = vec![None; n + 1];
    best[0] = Some(Vec::new());
    for end in 1..=n
    {
        best[end] = (0..end)
            .filter_map(|start| extend(&best[start], word, start, end))
            .min_by_key(Vec::len);
    }

    let pieces = best[n].take()?;
    ret_if!(pieces.len() < 2 || pieces.iter().all(|(kw, _)| kw.is_particle()), None);
    Some(pieces)
}
