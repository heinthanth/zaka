use unicode_ident::is_xid_continue;
use unicode_ident::is_xid_start;
use unicode_properties::UnicodeEmoji;

/// Whitespace characters. `U+200B` is included because the macOS
/// Burmese input method emits it.
pub const WHITESPACE: &[char] =
    &['\t', '\n', '\u{0B}', '\u{0C}', '\r', ' ', '\u{200B}'];

/// Burmese characters that end a word on their own.
const BURMESE_DELIMS: &[char] = &['၊', '။', '၏'];

/// Whether `ch` is whitespace.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` if `ch` is in [`WHITESPACE`].
pub fn is_whitespace(ch: char) -> bool { WHITESPACE.contains(&ch) }

/// Whether `ch` is a self-delimiting character: it is a word on its own.
///
/// These are `၊ ။ ၏` and the ASCII punctuation except `_`: the quotes and
/// operator characters, the backtick (raw identifiers), `.` (number
/// fractions), and the rest, so an invalid character is reported on its own
/// instead of spoiling the word next to it.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` if `ch` is self-delimiting.
pub fn is_self_delimiting(ch: char) -> bool
{
    BURMESE_DELIMS.contains(&ch) || (ch.is_ascii_punctuation() && ch != '_')
}

/// Whether `ch` is a Burmese digit.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` for `၀`–`၉`.
pub fn is_burmese_digit(ch: char) -> bool { ('၀'..='၉').contains(&ch) }

/// Whether `ch` is a Burmese character that can be part of an identifier.
///
/// The Burmese signs `၌ ၍ ၎` are not `XID_Continue`, but appear in common
/// words, so the whole Burmese block counts.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` for characters in `U+1000..=U+109F`.
fn is_burmese(ch: char) -> bool { ('\u{1000}'..='\u{109F}').contains(&ch) }

/// Whether `ch` is an emoji that can start an identifier.
///
/// ASCII characters with the Emoji flag (`0`–`9`, `#`, `*`) are left out.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` for non-ASCII emoji characters.
fn is_emoji_start(ch: char) -> bool { !ch.is_ascii() && ch.is_emoji_char() }

/// Whether `ch` can continue an emoji in an identifier: emoji characters and
/// emoji components (ZWJ, variation selectors, skin tones, tags).
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` for non-ASCII emoji characters and components.
fn is_emoji_continue(ch: char) -> bool
{
    !ch.is_ascii() && ch.is_emoji_char_or_emoji_component()
}

/// Whether `ch` can start an identifier.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` for `XID_Start`, `_`, emoji and Burmese characters other than
/// digits.
pub fn is_ident_start(ch: char) -> bool
{
    is_xid_start(ch)
        || ch == '_'
        || is_emoji_start(ch)
        || (is_burmese(ch) && !is_burmese_digit(ch))
}

/// Whether `ch` can continue an identifier.
///
/// # Arguments
///
/// * `ch` - The character to check.
///
/// # Returns
///
/// `true` for `XID_Continue`, `_`, emoji (with components) and Burmese
/// characters.
pub fn is_ident_continue(ch: char) -> bool
{
    is_xid_continue(ch) || ch == '_' || is_emoji_continue(ch) || is_burmese(ch)
}

/// Whether `text` is a whole identifier.
///
/// # Arguments
///
/// * `text` - The word to check.
///
/// # Returns
///
/// `true` if the first character passes [`is_ident_start`] and the rest pass
/// [`is_ident_continue`].
pub fn is_ident(text: &str) -> bool
{
    let mut chars = text.chars();
    let Some(first) = chars.next()
    else
    {
        return false;
    };
    is_ident_start(first) && chars.all(is_ident_continue)
}
