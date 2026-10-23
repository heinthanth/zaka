use zakac_i18n::t;
use zakac_lexer::phrases::PHRASES;
use zakac_lexer::reserved::RESERVED;
use zakac_lexer::token::Kw;

/// Put text in double quotes.
///
/// # Arguments
///
/// * `text` - The text.
///
/// # Returns
///
/// The quoted text.
pub(crate) fn quote(text: &str) -> String { format!("\"{text}\"") }

/// Spelling of a reserved word or joined phrase.
///
/// # Arguments
///
/// * `kw` - The reserved word.
///
/// # Returns
///
/// The first spelling in the reserved table; a phrase is spelled as its
/// pieces with spaces between them.
pub(crate) fn kw_text(kw: Kw) -> String
{
    let word =
        RESERVED.iter().find(|(_, k)| *k == kw).map(|(text, _)| text.to_string());
    word.or_else(|| phrase_text(kw)).unwrap_or_default()
}

/// Spelling of a joined phrase.
///
/// # Arguments
///
/// * `kw` - The joined reserved word.
///
/// # Returns
///
/// The pieces joined with spaces, or `None` when `kw` is no phrase.
fn phrase_text(kw: Kw) -> Option<String>
{
    let (pieces, _) = PHRASES.iter().find(|(_, k)| *k == kw)?;
    Some(pieces.iter().map(|piece| kw_text(*piece)).collect::<Vec<_>>().join(" "))
}

/// Join alternatives for an "expected" message.
///
/// # Arguments
///
/// * `items` - The alternatives, quoted.
///
/// # Returns
///
/// The alternatives joined with the localized "or".
pub(crate) fn one_of(items: &[String]) -> String { items.join(&t!("parser.or")) }
