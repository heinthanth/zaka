//! The terminal that standard error goes to, and how it draws Burmese text.
//!
//! Each kind of terminal has a check in [`CHECKS`]; the first check that
//! knows the terminal tells how it draws. A new terminal needs a check of its
//! own, and a [`Drawing`] for it when none of these fits.

use std::sync::OnceLock;

use bon::Builder;

use crate::flow::Flow;
use crate::flow::Part;
use crate::flow::Reach;

/// How a terminal draws Burmese text.
pub enum Drawing
{
    /// On the cell grid, one cell per code point; zero-width signs take none.
    Codepoint,
    /// On the cell grid, cells per grapheme cluster (mode 2027).
    Cluster,
    /// At the natural width of its font, flowing the row to make room.
    Flow(Flow),
}

/// A terminal that a check knows.
#[derive(Builder)]
pub struct Terminal
{
    /// Its name, for the reader.
    pub name: &'static str,
    /// How it draws Burmese text.
    pub drawing: Drawing,
}

/// The checks, in order: each returns the terminal when it knows it. The
/// checks of named terminals come first; measuring comes last, since it can't
/// tell a terminal that draws Burmese text off its grid.
const CHECKS: &[fn() -> Option<Terminal>] =
    &[crate::ghostty::probe, crate::measure::probe];

/// The terminal, once found.
static FOUND: OnceLock<Option<Terminal>> = OnceLock::new();

impl Terminal
{
    /// Tell which cells of a caret row sit under a part of the row above,
    /// when the terminal draws Burmese text off its cell grid.
    ///
    /// # Arguments
    ///
    /// * `parts` - The parts of the row, in order.
    /// * `marked` - Index of the part.
    ///
    /// # Returns
    ///
    /// The reach; `None` for a terminal that draws on the grid, where the
    /// cells of the text tell it, and for a row that wraps.
    pub fn reach(&self, parts: &[Part], marked: usize) -> Option<Reach>
    {
        let Drawing::Flow(flow) = &self.drawing
        else
        {
            return None;
        };
        flow.reach(parts, marked)
    }
}

/// The terminal that standard error goes to, when a check knows it; the
/// checks run once.
///
/// # Returns
///
/// The terminal, or `None`.
pub fn detect() -> Option<&'static Terminal>
{
    FOUND.get_or_init(|| CHECKS.iter().find_map(|check| check())).as_ref()
}
