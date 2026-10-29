use zakac_diagnostics::Code;
use zakac_diagnostics::Diagnostic;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;

use crate::flow::Flow;

pub(crate) mod bounds;
pub(crate) mod calls;
pub(crate) mod escape;
pub(crate) mod leaks;
pub(crate) mod lists;
pub(crate) mod loans;
pub(crate) mod moves;
pub(crate) mod nullness;
pub(crate) mod static_rules;

/// Keep a diagnostic when the flow runs its report pass, once for each
/// code, span and message, so owners lost at one exit each get theirs;
/// when the pass runs again over the loops and so does not
/// record, keep a read of a moved-out owner alone.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `found` - The diagnostic.
pub(crate) fn report(flow: &mut Flow, found: Diagnostic)
{
    ret_if!(!flow.reporting);
    ret_if!(!flow.recording() && found.code != Code::MovedOut);
    let seen = |old: &Diagnostic| {
        (old.code, old.span, &old.message) == (found.code, found.span, &found.message)
    };
    do_if!(!flow.diags.iter().any(seen), flow.diags.push(found));
}

/// The name a slot was declared with.
///
/// # Arguments
///
/// * `flow` - The flow.
/// * `place` - The slot.
///
/// # Returns
///
/// The name; blank for a slot out of range.
pub(crate) fn slot_name<'m>(flow: &Flow<'m>, place: Place) -> &'m str
{
    let slot = match place
    {
        Place::Global(n) => flow.module.globals.get(n as usize),
        Place::Local(n) => flow.routine.locals.get(n as usize),
    };
    slot.map_or("", |s| s.name.as_str())
}
