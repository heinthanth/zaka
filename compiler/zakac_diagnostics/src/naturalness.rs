use zakac_i18n::t;
use zakac_span::span::Span;

use crate::code::Code;
use crate::diagnostic::Diagnostic;

/// A negative verb before `သို့မဟုတ်` carries `သည်`.
///
/// # Arguments
///
/// * `span` - The verb, with its particle.
/// * `verb` - The verb text.
///
/// # Returns
///
/// The `ZK0501` diagnostic, with the `ပါ` spelling as a hint.
pub fn negative_before_or(span: Span, verb: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0501.message", v = verb);
    let help = t!("codes.ZK0501.help", v = verb);
    Diagnostic::new(Code::NegativeBeforeOr, msg, span).with_help(help)
}

/// A verb before an attached `လျှင်` or `နေသရွေ့` carries a particle.
///
/// # Arguments
///
/// * `span` - The verb, with its particle.
/// * `verb` - The verb text.
/// * `particle` - The particle found.
/// * `next` - The attached word that follows.
///
/// # Returns
///
/// The `ZK0502` diagnostic, with the bare spelling as a hint.
pub fn particle_before_attached(
    span: Span, verb: &str, particle: &str, next: &str,
) -> Diagnostic
{
    let msg = t!("codes.ZK0502.message", v = verb, p = particle, n = next);
    let help = t!("codes.ZK0502.help", v = verb, n = next);
    Diagnostic::new(Code::ParticleBeforeAttached, msg, span).with_help(help)
}

/// `မဟုတ်` carries `သည်`.
///
/// # Arguments
///
/// * `span` - The word, with its particle.
/// * `particle` - The particle found.
///
/// # Returns
///
/// The `ZK0503` diagnostic, with the `ပါ` spelling as a hint.
pub fn not_with_thi(span: Span, particle: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0503.message", p = particle);
    Diagnostic::new(Code::NotWithThi, msg, span).with_help(t!("codes.ZK0503.help"))
}

/// A verb before `ဖြစ်၍` carries `ပါ`.
///
/// # Arguments
///
/// * `span` - The verb, with its particle.
/// * `verb` - The verb text.
/// * `particle` - The particle found.
///
/// # Returns
///
/// The `ZK0504` diagnostic, with the `သည်` spelling as a hint.
pub fn pa_before_and(span: Span, verb: &str, particle: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0504.message", v = verb, p = particle);
    let help = t!("codes.ZK0504.help", v = verb);
    Diagnostic::new(Code::PaBeforeAnd, msg, span).with_help(help)
}

/// A sentence or a string is printed with `ကို`.
///
/// # Arguments
///
/// * `span` - The printed operand.
///
/// # Returns
///
/// The `ZK0505` diagnostic, with the `ဟု` spelling as a hint.
pub fn print_sentence_with_ko(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0505.message");
    Diagnostic::new(Code::PrintSentenceWithKo, msg, span)
        .with_help(t!("codes.ZK0505.help"))
}

/// A plain value is printed with `ဟု`.
///
/// # Arguments
///
/// * `span` - The printed operand.
///
/// # Returns
///
/// The `ZK0506` diagnostic, with the `ကို` spelling as a hint.
pub fn print_value_with_hu(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0506.message");
    Diagnostic::new(Code::PrintValueWithHu, msg, span)
        .with_help(t!("codes.ZK0506.help"))
}

/// Parentheses around an expression that stands alone in its place.
///
/// # Arguments
///
/// * `span` - The group, parentheses included.
///
/// # Returns
///
/// The `ZK0507` diagnostic.
pub fn redundant_group(span: Span) -> Diagnostic
{
    let msg = t!("codes.ZK0507.message");
    Diagnostic::new(Code::RedundantGroup, msg, span).with_help(t!("codes.ZK0507.help"))
}

/// A clause used as a value ends in `ပါ`.
///
/// # Arguments
///
/// * `span` - The clause.
/// * `verb` - The closing verb text.
/// * `particle` - The particle found.
///
/// # Returns
///
/// The `ZK0508` diagnostic, with the `သည်` spelling as a hint.
pub fn pa_in_value(span: Span, verb: &str, particle: &str) -> Diagnostic
{
    let msg = t!("codes.ZK0508.message", v = verb, p = particle);
    let help = t!("codes.ZK0508.help", v = verb);
    Diagnostic::new(Code::PaInValue, msg, span).with_help(help)
}
