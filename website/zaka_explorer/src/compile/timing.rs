//! How long the phases of a compile take, and how a time reads.

use std::time::Duration;

use bon::Builder;
use web_time::Instant;
use zakac_rust_sugars::ret_if;

use crate::compile::Phase;

/// How long each phase of a compile took; `None` for a phase that did not run.
/// The run has no time here: the debugger session keeps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Builder)]
pub struct PhaseTimes
{
    /// The segmenter.
    pub segment: Option<Duration>,
    /// The lexer.
    pub lex: Option<Duration>,
    /// The parser.
    pub parse: Option<Duration>,
    /// The naturalness and the semantic checks.
    pub check: Option<Duration>,
    /// Lowering to the IR and folding it.
    pub lower: Option<Duration>,
    /// The ownership check.
    pub borrow_check: Option<Duration>,
    /// Code generation.
    pub codegen: Option<Duration>,
}

impl PhaseTimes
{
    /// The time of a phase.
    ///
    /// # Arguments
    ///
    /// * `phase` - The phase.
    ///
    /// # Returns
    ///
    /// The time, or `None` for a phase that did not run and for the run.
    pub fn of(&self, phase: Phase) -> Option<Duration>
    {
        match phase
        {
            Phase::Segment => self.segment,
            Phase::Lex => self.lex,
            Phase::Parse => self.parse,
            Phase::Check => self.check,
            Phase::Lower => self.lower,
            Phase::BorrowCheck => self.borrow_check,
            Phase::Codegen => self.codegen,
            Phase::Run => None,
        }
    }

    /// The time of the whole compile: the sum of the phases that ran.
    ///
    /// # Returns
    ///
    /// The time.
    pub fn total(&self) -> Duration
    {
        let phases = [self.segment, self.lex, self.parse, self.check, self.lower];
        phases.into_iter().chain([self.borrow_check, self.codegen]).flatten().sum()
    }
}

/// Run a step of work and measure it.
///
/// # Arguments
///
/// * `work` - The work.
///
/// # Returns
///
/// What the work gave, and how long it took.
pub fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration)
{
    let start = Instant::now();
    let done = work();
    (done, start.elapsed())
}

/// A time as the pipeline and the run bar show it: µs under a millisecond,
/// ms under a second, then seconds, with two or three digits.
///
/// # Arguments
///
/// * `time` - The time.
///
/// # Returns
///
/// The text, such as `420 µs`, `1.24 ms`, `12.4 ms`, `240 ms` or `1.24 s`.
pub fn time_text(time: Duration) -> String
{
    let micros = time.as_secs_f64() * 1e6;
    ret_if!(micros < 1e3, format!("{micros:.0} µs"));
    let millis = micros / 1e3;
    ret_if!(millis < 10.0, format!("{millis:.2} ms"));
    ret_if!(millis < 100.0, format!("{millis:.1} ms"));
    ret_if!(millis < 1e3, format!("{millis:.0} ms"));
    format!("{:.2} s", millis / 1e3)
}
