use std::collections::HashMap;

use zakac_diagnostics::Diagnostic;
use zakac_diagnostics::fold as fd;
use zakac_rust_sugars::ret_if;

use crate::fold::blocker;
use crate::fold::fold_each;
use crate::fold::fold_module_each;
use crate::fold::is_known;
use crate::ir::Access;
use crate::ir::Expr;
use crate::ir::Module;
use crate::ir::Place;
use crate::ir::Slot;
use crate::ir::Stmt;
use crate::ir::each_stmt;

/// The value of a constant at its function (`None` for the top-level
/// statements) and slot.
pub(crate) type Statics = HashMap<(Option<u32>, Place), Expr>;

/// What the compile-time step works out: the static value of each constant
/// that must be static data, and the diagnostics of each value the
/// language needs at compile time and cannot have.
#[derive(Debug, Default)]
pub(crate) struct Known
{
    /// The static value of each constant that holds text or a plain list.
    pub(crate) values: Statics,
    /// The `ZK0604` and `ZK0606` diagnostics.
    pub(crate) diags: Vec<Diagnostic>,
}

/// Work out the values the language needs at compile time, whatever the
/// fold options of the build hold: each constant that holds text or a plain
/// list is static data (`ZK0606` when its value does not fold), and each
/// variable or constant of an imported file starts with a known value
/// (`ZK0604`). A duplicate of the module is folded with each folding on, so
/// the options change how the IR shows a value, never whether a program
/// compiles.
///
/// # Arguments
///
/// * `module` - The module, before folding.
/// * `platform_bits` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// The static values and the diagnostics.
pub(crate) fn comptime(module: &Module, platform_bits: u16) -> Known
{
    let mut known =
        Known { diags: unknown_inits(module, platform_bits), ..Known::default() };
    ret_if!(!has_static_consts(module), known);
    let mut folded = module.clone();
    fold_module_each(&mut folded, platform_bits);
    let mut keep = |func: Option<u32>, slots: &[Slot], s: &Stmt| {
        let Stmt::Set { place, value, .. } = s
        else
        {
            return;
        };
        let slot = slot_of(&module.globals, slots, *place);
        ret_if!(!slot.is_some_and(must_be_static));
        ret_if!(
            is_known(value),
            _ = known.values.insert((func, *place), value.clone())
        );
        known.diags.push(fd::const_not_known(blocker(value)));
    };
    each_stmt(&folded.main, &mut |s| keep(None, &[], s));
    for (i, func) in folded.funcs.iter().enumerate()
    {
        each_stmt(&func.block, &mut |s| keep(Some(i as u32), &func.locals, s));
    }
    known
}

/// Whether a module has a constant that must be static data.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// `true` when it has one.
fn has_static_consts(module: &Module) -> bool
{
    let locals = module.funcs.iter().flat_map(|f| &f.locals);
    module.globals.iter().chain(locals).any(must_be_static)
}

/// Whether a slot is a constant that must be static data: one that holds
/// text or a plain list, alone or inside a list or a struct.
///
/// # Arguments
///
/// * `slot` - The slot.
///
/// # Returns
///
/// `true` for such a constant.
fn must_be_static(slot: &Slot) -> bool
{
    slot.access == Access::Const && slot.kind.holds_text()
}

/// The slot a place names.
///
/// # Arguments
///
/// * `globals` - The global slots.
/// * `locals` - The local slots of the function the place is in.
/// * `place` - The place.
///
/// # Returns
///
/// The slot, or `None` for an index past the table.
fn slot_of<'m>(
    globals: &'m [Slot], locals: &'m [Slot], place: Place,
) -> Option<&'m Slot>
{
    match place
    {
        Place::Global(g) => globals.get(g as usize),
        Place::Local(l) => locals.get(l as usize),
    }
}

/// Report each variable or constant of an imported file whose starting
/// value is not known at compile time. A duplicate of the statements that
/// set them is folded with each folding on.
///
/// # Arguments
///
/// * `module` - The module, before folding.
/// * `platform_bits` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// One `ZK0604` diagnostic per such starting value.
fn unknown_inits(module: &Module, platform_bits: u16) -> Vec<Diagnostic>
{
    let mut inits = module.main[..module.imported_inits as usize].to_vec();
    fold_each(&mut inits, &module.globals, platform_bits);
    inits.iter().filter_map(|s| unknown_init(s, &module.globals)).collect()
}

/// Report a variable or constant of an imported file whose starting value
/// did not fold to a value known at compile time.
///
/// # Arguments
///
/// * `stmt` - The folded statement that sets it.
/// * `globals` - The global slots.
///
/// # Returns
///
/// The `ZK0604` diagnostic, or `None` for a known value.
fn unknown_init(stmt: &Stmt, globals: &[Slot]) -> Option<Diagnostic>
{
    let Stmt::Set { place: Place::Global(g), value, span } = stmt
    else
    {
        return None;
    };
    ret_if!(is_known(value), None);
    Some(fd::global_init_not_constant(*span, &globals[*g as usize].name))
}
