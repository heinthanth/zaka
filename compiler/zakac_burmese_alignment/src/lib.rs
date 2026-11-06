//! Caret rows under Burmese text.
//!
//! Terminals do not agree on how wide Burmese text is drawn. Some count one
//! cell per code point, some count grapheme clusters, and some draw the text
//! at the natural width of its font and push the rest of the row right, so a
//! row of carets counted in cells falls short of the text it points at.
//!
//! [`terminal::detect`] finds the terminal that standard error goes to and
//! tells how it draws; [`flow`] places a row as a flowing terminal does.

pub mod flow;
#[cfg(unix)]
pub mod fonts;
pub mod ghostty;
pub mod measure;
pub mod terminal;
