use zakac_i18n::t;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// A kind the C target cannot build: C has no portable kind for it, and no
/// print that writes it as the VM does.
///
/// # Arguments
///
/// * `span` - Where the kind is used.
/// * `kind` - The kind, as text.
///
/// # Returns
///
/// The `ZK0801` diagnostic, with help on the kinds to use instead.
pub fn kind_not_for_c(span: Span, kind: &str) -> Diagnostic
{
    let diag = Diagnostic::new(
        Code::KindNotForC,
        t!("codes.ZK0801.message", kind = kind),
        span,
    );
    diag.with_help(t!("codes.ZK0801.help"))
}
