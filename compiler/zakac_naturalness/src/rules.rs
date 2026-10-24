use zakac_ast::expr::NextCtx;
use zakac_ast::verb::Slot;
use zakac_diagnostics::Code;
use zakac_lexer::token::Particle;
use zakac_rust_sugars::ret_false;

use crate::site::FinalSite;
use crate::site::FinalWord;
use crate::site::GroupSite;
use crate::site::Inner;
use crate::site::Place;
use crate::site::PrintSite;
use crate::site::Shape;
use crate::site::Site;
use crate::site::ValueSite;

/// One particle rule: the code it reports and the test it applies.
#[derive(Debug, Clone, Copy)]
pub struct Rule
{
    /// The diagnostic code.
    pub code: Code,
    /// Whether the rule fires at a site.
    pub fires: fn(&Site) -> bool,
}

/// The rules, in order; the first that fires at a site wins.
///
/// To add a rule, add a row here, a `ZK05xx` code with its texts in
/// `zakac_diagnostics`, and a constructor call in the checker.
pub static RULES: &[Rule] = &[
    Rule { code: Code::NegativeBeforeOr, fires: negative_before_or },
    Rule { code: Code::ParticleBeforeAttached, fires: particle_before_attached },
    Rule { code: Code::NotWithThi, fires: not_with_thi },
    Rule { code: Code::PaBeforeAnd, fires: pa_before_and },
    Rule { code: Code::PrintSentenceWithKo, fires: print_sentence_with_ko },
    Rule { code: Code::PrintValueWithHu, fires: print_value_with_hu },
    Rule { code: Code::RedundantGroup, fires: redundant_group },
    Rule { code: Code::PaInValue, fires: pa_in_value },
];

/// The first rule that fires at a site.
///
/// # Arguments
///
/// * `site` - The site.
///
/// # Returns
///
/// The code to report, or `None` when the site is idiomatic.
pub fn first_match(site: &Site) -> Option<Code>
{
    RULES.iter().find(|rule| (rule.fires)(site)).map(|rule| rule.code)
}

/// The clause-final site, when the site is one.
fn as_final(site: &Site) -> Option<&FinalSite>
{
    match site
    {
        Site::Final(n) => Some(n),
        _ => None,
    }
}

/// The print site, when the site is one.
fn as_print(site: &Site) -> Option<&PrintSite>
{
    match site
    {
        Site::Print(n) => Some(n),
        _ => None,
    }
}

/// The group site, when the site is one.
fn as_group(site: &Site) -> Option<&GroupSite>
{
    match site
    {
        Site::Group(n) => Some(n),
        _ => None,
    }
}

/// A negative word before `သို့မဟုတ်` carries `သည်`.
fn negative_before_or(site: &Site) -> bool
{
    let Some(n) = as_final(site)
    else
    {
        return false;
    };
    n.word.negative() && n.next == NextCtx::Or && n.particle == Particle::Thi
}

/// A word before an attached `လျှင်` or `နေသရွေ့` carries a particle.
fn particle_before_attached(site: &Site) -> bool
{
    let Some(n) = as_final(site)
    else
    {
        return false;
    };
    let attached = matches!(n.next, NextCtx::Then | NextCtx::While);
    attached && n.particle != Particle::None
}

/// `မဟုတ်` carries `သည်` or `ပါသည်`.
fn not_with_thi(site: &Site) -> bool
{
    let Some(n) = as_final(site)
    else
    {
        return false;
    };
    n.word == FinalWord::Not && matches!(n.particle, Particle::Thi | Particle::PaThi)
}

/// A word before `ဖြစ်၍` carries `ပါ` or `ပါသည်`.
fn pa_before_and(site: &Site) -> bool
{
    let Some(n) = as_final(site)
    else
    {
        return false;
    };
    n.next == NextCtx::And && matches!(n.particle, Particle::Pa | Particle::PaThi)
}

/// A sentence or a string is printed with `ကို`.
fn print_sentence_with_ko(site: &Site) -> bool
{
    let Some(n) = as_print(site)
    else
    {
        return false;
    };
    ret_false!(n.written != Slot::Ko);
    matches!(n.shape, Shape::Sentence | Shape::Str)
}

/// A plain value is printed with `ဟု`.
fn print_value_with_hu(site: &Site) -> bool
{
    let Some(n) = as_print(site)
    else
    {
        return false;
    };
    n.written == Slot::Hu && n.shape == Shape::Value
}

/// Parentheses that change nothing: nested ones, a whole condition, or a
/// whole operand that holds no logical expression.
fn redundant_group(site: &Site) -> bool
{
    let Some(n) = as_group(site)
    else
    {
        return false;
    };
    match n.place
    {
        Place::Nested | Place::Condition => true,
        Place::Operand => n.inner == Inner::Plain,
    }
}

/// The value site, when the site is one.
fn as_value(site: &Site) -> Option<&ValueSite>
{
    match site
    {
        Site::Value(n) => Some(n),
        _ => None,
    }
}

/// A clause used as a value ends in `ပါ` or `ပါသည်`.
fn pa_in_value(site: &Site) -> bool
{
    let Some(n) = as_value(site)
    else
    {
        return false;
    };
    matches!(n.tail, Particle::Pa | Particle::PaThi)
}
