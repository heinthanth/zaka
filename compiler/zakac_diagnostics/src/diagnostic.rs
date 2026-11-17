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

/// Where a fix comes from: a rule of the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(AsRefStr, IntoStaticStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FixOrigin
{
    /// A rule of the compiler.
    Rule,
}

/// One change of a fix: the text that takes the place of a source region. A
/// blank region inserts the text; a blank text removes the region.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Edit
{
    /// Source region the text replaces.
    pub span: Span,
    /// The new text.
    #[builder(into)]
    pub text: String,
}

/// One fix for a diagnostic. The compiler never applies it: the reader
/// does, through the editor or with a hand edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
pub struct Fix
{
    /// Localized text that tells what the fix does.
    #[builder(into)]
    pub message: String,
    /// The changes, in source order; no two overlap.
    pub edits: Vec<Edit>,
    /// Where the fix comes from.
    pub origin: FixOrigin,
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
    /// Fixes for the problem; the first is the likeliest.
    #[builder(default)]
    #[serde(default)]
    pub fixes: Vec<Fix>,
}

impl Serialize for Diagnostic
{
    /// Write each field. The text artifacts leave out `labels` and `fixes`
    /// when nothing is in them, so older goldens are unchanged. The raw
    /// artifacts are not self-describing: all fields are written.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>
    {
        /// The number of fields, `labels` and `fixes` included.
        const FIELDS: usize = 7;
        let text = serializer.is_human_readable();
        let skip_labels = text && self.labels.is_empty();
        let skip_fixes = text && self.fixes.is_empty();
        let n = FIELDS - usize::from(skip_labels) - usize::from(skip_fixes);
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
        tern!(
            skip_fixes,
            state.skip_field("fixes")?,
            state.serialize_field("fixes", &self.fixes)?
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

    /// Attach a rule fix that puts `text` in place of one region.
    ///
    /// # Arguments
    ///
    /// * `message` - The localized text that tells what the fix does.
    /// * `span` - The region to replace; a blank region inserts.
    /// * `text` - The new text; a blank text removes the region.
    ///
    /// # Returns
    ///
    /// The diagnostic with the fix added after the existing ones.
    pub fn with_fix(
        mut self, message: impl Into<String>, span: Span, text: impl Into<String>,
    ) -> Self
    {
        let edit = Edit::builder().span(span).text(text).build();
        let fix = Fix::builder().message(message).edits(vec![edit]);
        self.fixes.push(fix.origin(FixOrigin::Rule).build());
        self
    }

    /// Turn the help text into a rule fix that puts `text` in place of one
    /// region. The help tells what the fix does, so it becomes the message of
    /// the fix and is not shown twice.
    ///
    /// # Arguments
    ///
    /// * `span` - The region to replace; a blank region inserts.
    /// * `text` - The new text; a blank text removes the region.
    ///
    /// # Returns
    ///
    /// The diagnostic with no help text and the fix added.
    pub fn help_as_fix(mut self, span: Span, text: impl Into<String>) -> Self
    {
        let message = self.help.take().unwrap_or_default();
        self.with_fix(message, span, text)
    }

    /// Move the main span and the span of each label and fix edit, as when
    /// spans are made relative to a file or absolute again.
    ///
    /// # Arguments
    ///
    /// * `f` - Maps one span.
    pub fn map_spans(&mut self, f: &dyn Fn(Span) -> Span)
    {
        self.span = f(self.span);
        self.labels.iter_mut().for_each(|label| label.span = f(label.span));
        let edits = self.fixes.iter_mut().flat_map(|fix| fix.edits.iter_mut());
        edits.for_each(|edit| edit.span = f(edit.span));
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
