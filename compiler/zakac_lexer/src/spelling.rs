//! Words with two spellings that mean one thing. `ဒေတာ` and `အချက်အလက်` are
//! one word in each reserved word and each name, and
//! `အချက်အလက်` is the spelling the compiler keeps. Text in string literals
//! and comments keeps the spelling written.

use std::borrow::Cow;

use zakac_rust_sugars::tern;

/// The spelling the compiler keeps.
pub const DATA: &str = "အချက်အလက်";

/// The loan word that means the same.
pub const DATA_LOAN: &str = "ဒေတာ";

/// The spelling the compiler keeps for a word or a name.
///
/// # Arguments
///
/// * `text` - The word or the name, in normalized spelling.
///
/// # Returns
///
/// The text with each `ဒေတာ` written as `အချက်အလက်`; borrowed when there is
/// none.
pub fn canonical(text: &str) -> Cow<'_, str>
{
    let loan = text.contains(DATA_LOAN);
    tern!(loan, Cow::Owned(text.replace(DATA_LOAN, DATA)), Cow::Borrowed(text))
}

/// Each form a name can be written in, for a match against source text that
/// has not been made canonical: the text itself, and the text with each
/// spelling turned into the other.
///
/// # Arguments
///
/// * `text` - The name as written.
///
/// # Returns
///
/// The spellings, the written one first, with no repeat.
pub fn spellings(text: &str) -> Vec<String>
{
    // shortcut: a name that holds both spellings matches three of its four
    // forms; list each mix if a program ever needs it.
    let mut all = vec![text.to_owned()];
    let turned = [text.replace(DATA_LOAN, DATA), text.replace(DATA, DATA_LOAN)];
    all.extend(turned.into_iter().filter(|t| t != text));
    all.dedup();
    all
}
