//! The asker of a run on a terminal: it writes the question for a right the
//! run lacks to standard error and reads the answer from standard input, as
//! Deno asks.

use std::io::BufRead;
use std::io::IsTerminal;
use std::io::Write;

use zaka_host::Answer;
use zaka_host::Asker;
use zaka_host::Right;
use zakac_i18n::t;

use crate::trap::denied_what;

/// The asker of a run on a terminal.
#[derive(Debug, Clone, Copy, Default)]
pub struct TermAsker;

impl TermAsker
{
    /// The asker, when standard input and standard error are a terminal.
    ///
    /// # Returns
    ///
    /// The asker, or `None` for a run with no person to ask.
    pub fn on_terminal() -> Option<Box<dyn Asker>>
    {
        let terminal =
            std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
        terminal.then(|| Box::new(TermAsker) as Box<dyn Asker>)
    }
}

impl Asker for TermAsker
{
    fn ask(&mut self, right: Right, subject: &str) -> Answer
    {
        let question = t!("vm.permission.ask", what = denied_what(right, subject));
        let mut err = std::io::stderr();
        _ = write!(err, "{question}");
        _ = err.flush();
        let mut line = String::new();
        _ = std::io::stdin().lock().read_line(&mut line);
        Answer::typed(&line)
    }
}
