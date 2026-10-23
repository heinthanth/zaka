use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use strum::AsRefStr;
use zakac_span::span::Span;

use crate::code::Code;

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Level
{
    /// The input is wrong; the phase output is not usable as is.
    Error,
    /// The input is accepted but looks wrong.
    Warning,
    /// Extra information.
    Note,
}

/// Whether a list of diagnostics holds an error.
///
/// # Arguments
///
/// * `diags` - The diagnostics to check.
///
/// # Returns
///
/// `true` if one of the diagnostics is an error.
pub fn has_errors(diags: &[Diagnostic]) -> bool
{
    diags.iter().any(Diagnostic::is_error)
}

/// A message about a region of source code, produced in a compiler phase.
///
/// `message` and `help` are localized where the diagnostic is created, from the
/// texts in the catalogue (`locales/codes.toml`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Diagnostic
{
    /// The stable code.
    pub code: Code,
    /// How serious the diagnostic is; the code's level unless overridden.
    pub level: Level,
    /// Localized message.
    #[builder(into)]
    pub message: String,
    /// Source region the diagnostic points at.
    pub span: Span,
    /// Localized hint on how to fix the problem.
    #[builder(into)]
    pub help: Option<String>,
}

impl Diagnostic
{
    /// Create a diagnostic at the code's level.
    ///
    /// # Arguments
    ///
    /// * `code` - The stable code.
    /// * `message` - The localized message.
    /// * `span` - The source region.
    ///
    /// # Returns
    ///
    /// The new diagnostic, with no help text.
    pub fn new(code: Code, message: impl Into<String>, span: Span) -> Self
    {
        Self::builder()
            .code(code)
            .level(code.level())
            .message(message)
            .span(span)
            .build()
    }

    /// Override the level.
    ///
    /// # Arguments
    ///
    /// * `level` - The new level.
    ///
    /// # Returns
    ///
    /// The diagnostic with the level set.
    pub fn with_level(mut self, level: Level) -> Self
    {
        self.level = level;
        self
    }

    /// Attach help text.
    ///
    /// # Arguments
    ///
    /// * `help` - The localized hint.
    ///
    /// # Returns
    ///
    /// The diagnostic with the help text set.
    pub fn with_help(mut self, help: impl Into<String>) -> Self
    {
        self.help = Some(help.into());
        self
    }

    /// Clone the diagnostic with a new span.
    ///
    /// # Arguments
    ///
    /// * `span` - The new source region.
    ///
    /// # Returns
    ///
    /// A clone of the diagnostic pointing at `span`.
    pub fn with_span(&self, span: Span) -> Self
    {
        let mut diag = self.clone();
        diag.span = span;
        diag
    }

    /// Whether this diagnostic is an error.
    ///
    /// # Returns
    ///
    /// `true` for [`Level::Error`], `false` otherwise.
    pub fn is_error(&self) -> bool { self.level == Level::Error }
}
