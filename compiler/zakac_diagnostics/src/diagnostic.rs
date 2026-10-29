use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;
use strum::AsRefStr;
use strum::IntoStaticStr;
use zakac_rust_sugars::tern;
use zakac_span::span::Span;

use crate::code::Code;

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr, IntoStaticStr)]
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

/// An extra source region with a short text, shown under the main
/// snippet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Label
{
    /// Source region the label points at.
    pub span: Span,
    /// Localized text shown next to the region.
    #[builder(into)]
    pub text: String,
}

/// A message about a region of source code, produced in a compiler phase.
///
/// `message` and `help` are localized where the diagnostic is created, from the
/// texts in the catalogue (`locales/codes.toml`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Builder)]
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
    /// Extra regions, shown in order under the main snippet.
    #[builder(default)]
    #[serde(default)]
    pub labels: Vec<Label>,
}

impl Serialize for Diagnostic
{
    /// Write each field. The text artifacts leave out `labels` when it holds
    /// no label, so older goldens are unchanged. The raw artifacts are not
    /// self-describing: all fields are written.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        /// The number of fields, `labels` included.
        const FIELDS: usize = 6;
        let skip_labels = serializer.is_human_readable() && self.labels.is_empty();
        let n = FIELDS - usize::from(skip_labels);
        let mut state = serializer.serialize_struct("Diagnostic", n)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field("level", &self.level)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("span", &self.span)?;
        state.serialize_field("help", &self.help)?;
        tern!(
            skip_labels,
            state.skip_field("labels")?,
            state.serialize_field("labels", &self.labels)?
        );
        state.end()
    }
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

    /// Attach an extra label.
    ///
    /// # Arguments
    ///
    /// * `span` - The source region the label points at.
    /// * `text` - The localized label text.
    ///
    /// # Returns
    ///
    /// The diagnostic with the label added after the existing ones.
    pub fn with_label(mut self, span: Span, text: impl Into<String>) -> Self
    {
        self.labels.push(Label::builder().span(span).text(text).build());
        self
    }

    /// Move the main span and the span of each label, as when spans are
    /// made relative to a file or absolute again.
    ///
    /// # Arguments
    ///
    /// * `f` - Maps one span.
    pub fn map_spans(&mut self, f: &dyn Fn(Span) -> Span)
    {
        self.span = f(self.span);
        self.labels.iter_mut().for_each(|label| label.span = f(label.span));
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
