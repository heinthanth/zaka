//! The check for a terminal that no other check knows: measure how it counts
//! the cells of Burmese text.
//!
//! The check prints [`TEST_WORD`] hidden and asks the terminal where the
//! cursor went. A terminal that gives each code point its own cells moves the
//! cursor four columns; one that gives a grapheme cluster its cells (mode
//! 2027) moves it three. Then the check erases the word.
//!
//! Each step waits for the one before it to work, so a terminal that does
//! not answer sees nothing:
//!
//! 1. Ask for the cursor position before printing; with no answer within a
//!    tenth of a second, the check gives up.
//! 2. Print the word with the conceal attribute, so that it is not drawn.
//! 3. Ask again, move back to the first column and erase to the end of the
//!    line.
//!
//! A terminal that draws Burmese text off its grid still reports grid columns,
//! so a check of its own must come before this one.

use crate::terminal::Drawing;
use crate::terminal::Terminal;

/// The word the check prints: `ကျ` and `ေ` make one cluster of two cells, and
/// `ာ` a cluster of one, so the word takes three cells as clusters and four as
/// code points.
pub const TEST_WORD: &str = "ကျော";

/// Read the column out of a cursor position answer, `ESC [ row ; column R`.
///
/// # Arguments
///
/// * `answer` - The octets the terminal sent; the last answer in them counts.
///
/// # Returns
///
/// The 1-based column, or `None` when the octets hold no answer.
pub fn answer_column(answer: &[u8]) -> Option<u16>
{
    let text = std::str::from_utf8(answer).ok()?;
    let end = text.rfind('R')?;
    let start = text[..end].rfind('\u{1b}')?;
    let (_, col) = text[start..end].strip_prefix("\u{1b}[")?.split_once(';')?;
    col.parse().ok()
}

/// How a terminal draws Burmese text, from the columns [`TEST_WORD`] moved
/// the cursor.
///
/// # Arguments
///
/// * `moved` - The columns.
///
/// # Returns
///
/// The drawing, or `None` for a count that fits neither.
pub fn drawing_for(moved: i32) -> Option<Drawing>
{
    match moved
    {
        4 => Some(Drawing::Codepoint),
        3 => Some(Drawing::Cluster),
        _ => None,
    }
}

/// Measure the terminal that standard error goes to.
///
/// # Returns
///
/// The terminal, or `None` when standard error is not a terminal, the
/// terminal does not answer, or its count fits neither drawing.
pub fn probe() -> Option<Terminal>
{
    #[cfg(unix)]
    {
        unix::probe()
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(unix)]
mod unix
{
    use std::fs::File;
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::io::Write;

    use rustix::termios::LocalModes;
    use rustix::termios::OptionalActions;
    use rustix::termios::QueueSelector;
    use rustix::termios::SpecialCodeIndex;
    use rustix::termios::Termios;
    use rustix::termios::isatty;
    use rustix::termios::tcflush;
    use rustix::termios::tcgetattr;
    use rustix::termios::tcgetwinsize;
    use rustix::termios::tcsetattr;
    use zakac_rust_sugars::ret_if;

    use super::TEST_WORD;
    use super::answer_column;
    use super::drawing_for;
    use crate::terminal::Terminal;

    /// How long a read waits for the terminal, in tenths of a second.
    const WAIT_TENTHS: u8 = 1;

    /// The longest answer the check reads, in octets.
    const ANSWER_MAX: usize = 32;

    /// Puts the terminal back in its saved state when dropped.
    struct Restore
    {
        term: File,
        saved: Termios,
    }

    impl Drop for Restore
    {
        fn drop(&mut self)
        {
            _ = tcsetattr(&self.term, OptionalActions::Now, &self.saved);
        }
    }

    /// The check; see [`super::probe`].
    ///
    /// # Returns
    ///
    /// The terminal, or `None`.
    pub(super) fn probe() -> Option<Terminal>
    {
        ret_if!(!isatty(std::io::stderr()), None);
        let mut term =
            OpenOptions::new().read(true).write(true).open("/dev/tty").ok()?;
        let saved = tcgetattr(&term).ok()?;
        let _restore = Restore { term: term.try_clone().ok()?, saved: saved.clone() };
        let mut quiet = saved;
        quiet.local_modes.remove(LocalModes::ICANON | LocalModes::ECHO);
        quiet.special_codes[SpecialCodeIndex::VMIN] = 0;
        quiet.special_codes[SpecialCodeIndex::VTIME] = WAIT_TENTHS;
        tcsetattr(&term, OptionalActions::Now, &quiet).ok()?;
        let moved = measure(&mut term);
        // an answer that came after its wait would reach the shell
        _ = tcflush(&term, QueueSelector::IFlush);
        let drawing = drawing_for(moved?)?;
        Some(Terminal::builder().name("measured").drawing(drawing).build())
    }

    /// Print the test word hidden, measure how far it moved the cursor, and
    /// erase it.
    ///
    /// # Arguments
    ///
    /// * `term` - The terminal, with echo off and reads that wait.
    ///
    /// # Returns
    ///
    /// The columns, or `None` when the terminal does not answer or the word
    /// would not fit on the line.
    fn measure(term: &mut File) -> Option<i32>
    {
        let start = ask_column(term)?;
        let cols = tcgetwinsize(&*term).ok()?.ws_col;
        ret_if!(start + 4 > cols, None);
        write!(term, "\u{1b}[8m{TEST_WORD}\u{1b}[28m").ok()?;
        let end = ask_column(term);
        write!(term, "\u{1b}[{start}G\u{1b}[K").ok()?;
        term.flush().ok()?;
        Some(i32::from(end?) - i32::from(start))
    }

    /// Ask the terminal where the cursor is.
    ///
    /// # Arguments
    ///
    /// * `term` - The terminal, with echo off and reads that wait.
    ///
    /// # Returns
    ///
    /// The 1-based column, or `None` when no answer comes.
    fn ask_column(term: &mut File) -> Option<u16>
    {
        term.write_all(b"\x1b[6n").ok()?;
        term.flush().ok()?;
        let mut answer = Vec::new();
        let mut octet = [0_u8; 1];
        while !answer.ends_with(b"R")
        {
            let read = term.read(&mut octet).ok()?;
            ret_if!(read == 0 || answer.len() > ANSWER_MAX, None);
            answer.push(octet[0]);
        }
        answer_column(&answer)
    }
}
