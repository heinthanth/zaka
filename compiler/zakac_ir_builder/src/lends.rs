//! A slice of a plain list makes the list own its cells first, since a slice
//! can write them, and cells in static data or lent to a parameter must
//! never change. A slice that reads alone needs no such step, so a static
//! list lent to a reader keeps pointing at its static cells.
//!
//! The pass follows each slice through slots, calls and returns. A slice
//! that is written, kept where the pass cannot follow, or given back from a
//! function counts as written. The own step of a slice that reads alone goes
//! when each other own step on the same list sits in a write that ends the
//! loans into it, so no loan can outlive a move of its cells.

use std::collections::HashMap;
use std::collections::HashSet;

use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::extract;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::ir::Dest;
use crate::ir::Expr;
use crate::ir::ExprNode;
use crate::ir::FmtPiece;
use crate::ir::Module;
use crate::ir::Place;
use crate::ir::PrintValue;
use crate::ir::Stmt;

/// A slot: the function it belongs to, or `None` for a global.
type Spot = (Option<u32>, Place);

/// How a value is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Use
{
    /// It is read, never written.
    Read,
    /// It can be written, or it goes where the pass cannot follow.
    Written,
    /// It is kept in a slot.
    Kept(Spot),
    /// It is passed to a parameter: the function and the parameter number.
    Passed(u32, u32),
}

/// What the walk finds, and the lends it lets go.
#[derive(Debug, bon::Builder)]
struct Walk
{
    /// The function walked, or `None` for the top-level statements.
    scope: Option<u32>,
    /// Each use of a slice slot.
    #[builder(default)]
    uses: Vec<(Spot, Use)>,
    /// Each slice made of a plain list slot: the list and the use.
    #[builder(default)]
    lends: Vec<(Spot, Use)>,
    /// The count of own steps on each plain list slot outside a write.
    #[builder(default)]
    owns: HashMap<Spot, u32>,
    /// The plain list slots whose address the program takes.
    #[builder(default)]
    addressed: HashSet<Spot>,
    /// The slice slots that can be written.
    #[builder(default)]
    written: HashSet<Spot>,
    /// The plain list slots whose reading lends skip the own step; `None`
    /// while the first walk gathers the uses.
    freed: Option<HashSet<Spot>>,
}

/// Drop the own step of each slice of a plain list that reads alone.
///
/// # Arguments
///
/// * `module` - The module, changed in place.
pub(crate) fn free_read_lends(module: &mut Module)
{
    #[rustfmt::skip]
    let mut walk = Walk::builder().addressed(addressed_slots(module))
        .written(given_back(module)).build();
    walk.module(module);
    walk.solve();
    walk.freed = Some(walk.free_lists());
    walk.module(module);
}

/// The slots marked as addressed: each one a pointer can reach.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The slots.
fn addressed_slots(module: &Module) -> HashSet<Spot>
{
    let globals = module.globals.iter().enumerate();
    let globals = globals.filter(|(_, s)| s.addressed);
    let mut found: HashSet<Spot> =
        globals.map(|(n, _)| (None, Place::Global(n as u32))).collect();
    for (f, func) in module.funcs.iter().enumerate()
    {
        let locals = func.locals.iter().enumerate().filter(|(_, s)| s.addressed);
        found.extend(locals.map(|(n, _)| (Some(f as u32), Place::Local(n as u32))));
    }
    found
}

/// The slice parameters and the result slot of each function that gives
/// back a slice: the caller can write the slice it gets, so each of these
/// counts as written.
///
/// # Arguments
///
/// * `module` - The module.
///
/// # Returns
///
/// The slots.
fn given_back(module: &Module) -> HashSet<Spot>
{
    let mut found = HashSet::new();
    let returns = module.funcs.iter().enumerate();
    for (f, func) in
        returns.filter(|(_, func)| func.ret.is_some_and(DataKind::is_slice))
    {
        let scope = Some(f as u32);
        let params = func.locals.iter().take(func.params as usize).enumerate();
        let slices = params.filter(|(_, s)| s.kind.is_slice());
        found.extend(slices.map(|(n, _)| (scope, Place::Local(n as u32))));
        do_if!(let Some(r) = func.result, _ = found.insert((scope, Place::Local(r))));
    }
    found
}

/// The use of a value whose own use the walk does not track: an address
/// can be written through, each other value is read.
///
/// # Arguments
///
/// * `kind` - The kind of the value.
///
/// # Returns
///
/// The use.
fn value_use(kind: DataKind) -> Use
{
    tern!(kind.is_ptr() || kind.is_slice(), Use::Written, Use::Read)
}

/// The use of the cells of a list or a struct whose address is used so: a
/// cell read leaves them read, each other use can write them.
///
/// # Arguments
///
/// * `u` - The use of the address.
///
/// # Returns
///
/// The use of the cells.
fn cells_use(u: Use) -> Use { tern!(u == Use::Read, Use::Read, Use::Written) }

/// The plain list slot an own step reads.
///
/// # Arguments
///
/// * `e` - The value the own step takes.
///
/// # Returns
///
/// The slot, or `None` for a list read through an address.
fn plain_slot(e: &Expr) -> Option<Place>
{
    let ExprNode::Read(place) = e.node
    else
    {
        return None;
    };
    e.kind.is_seq().then_some(place)
}

impl Walk
{
    /// Walk each function, then the top-level statements.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    fn module(&mut self, module: &mut Module)
    {
        for (f, func) in module.funcs.iter_mut().enumerate()
        {
            self.scope = Some(f as u32);
            self.stmts(&mut func.block);
        }
        self.scope = None;
        self.stmts(&mut module.main);
    }

    /// Mark each slice slot that can be written, through the slots and the
    /// parameters it is kept in, until nothing changes.
    fn solve(&mut self)
    {
        let mut changed = true;
        while changed
        {
            let open = self
                .uses
                .iter()
                .filter(|(k, u)| !self.written.contains(k) && self.is_written(*u));
            let found: Vec<Spot> = open.map(|(k, _)| *k).collect();
            changed = !found.is_empty();
            self.written.extend(found);
        }
    }

    /// The plain list slots whose reading lends skip the own step: each
    /// own step on the slot outside a write is the step of a reading lend,
    /// and no pointer reaches the slot.
    ///
    /// # Returns
    ///
    /// The slots.
    fn free_lists(&self) -> HashSet<Spot>
    {
        let mut clean: HashMap<Spot, u32> = HashMap::new();
        let reads = self.lends.iter().filter(|(_, u)| !self.is_written(*u));
        do_for!((k, _) in reads, *clean.entry(*k).or_default() += 1);
        let free = |(k, n): &(Spot, u32)| {
            self.owns.get(k) == Some(n) && !self.addressed.contains(k)
        };
        clean.into_iter().filter(free).map(|(k, _)| k).collect()
    }

    /// Whether a use can write the value.
    ///
    /// # Arguments
    ///
    /// * `u` - The use.
    ///
    /// # Returns
    ///
    /// `true` when it can.
    fn is_written(&self, u: Use) -> bool
    {
        match u
        {
            Use::Read => false,
            Use::Written => true,
            Use::Kept(k) => self.written.contains(&k),
            Use::Passed(f, i) => self.written.contains(&(Some(f), Place::Local(i))),
        }
    }

    /// The spot of a slot in the function walked.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The spot.
    fn spot(&self, place: Place) -> Spot
    {
        tern!(matches!(place, Place::Global(_)), (None, place), (self.scope, place))
    }

    /// Walk statements.
    ///
    /// # Arguments
    ///
    /// * `stmts` - The statements.
    fn stmts(&mut self, stmts: &mut [Stmt])
    {
        do_for!(s in stmts, self.stmt(s));
    }

    /// Walk one statement.
    ///
    /// # Arguments
    ///
    /// * `s` - The statement.
    fn stmt(&mut self, s: &mut Stmt)
    {
        match s
        {
            Stmt::Set { place, value, .. } =>
            {
                let u = tern!(value.kind.is_slice(), Use::Kept(self.spot(*place)), {
                    value_use(value.kind)
                });
                self.expr(value, u, false);
            }
            Stmt::Print { value: PrintValue::Expr(e), .. } =>
            {
                self.expr(e, Use::Read, false)
            }
            Stmt::Halt { value: Some(value), .. } => self.expr(value, Use::Read, false),
            Stmt::Halt { value: None, .. } => (),
            Stmt::Print { .. } | Stmt::Break { .. } | Stmt::Continue { .. } =>
            {}
            Stmt::If { cond, then, els, .. } =>
            {
                self.expr(cond, Use::Read, false);
                self.stmts(then);
                self.stmts(els);
            }
            Stmt::Loop { init, cond, update, block, .. } =>
            {
                self.stmts(init);
                self.expr(cond, Use::Read, false);
                self.stmts(block);
                self.stmts(update);
            }
            Stmt::Counted { from, to, step, block, .. } =>
            {
                self.expr(from, Use::Read, false);
                self.expr(to, Use::Read, false);
                do_if!(let Some(step) = step, self.expr(step, Use::Read, false));
                self.stmts(block);
            }
            Stmt::Return { value, .. } =>
            {
                do_if!(let Some(v) = value, self.expr(v, Use::Written, false));
            }
            Stmt::Call { func, args, .. } => self.args(*func, args),
            Stmt::Store { addr, value, .. } =>
            {
                self.expr(addr, Use::Written, true);
                self.expr(value, value_use(value.kind), false);
            }
            Stmt::Alloc { dest, count, .. } | Stmt::Resize { dest, count, .. } =>
            {
                self.dest(dest, false);
                self.expr(count, Use::Read, false);
            }
            Stmt::Free { dest, .. } | Stmt::Drop { dest, .. } => self.dest(dest, false),
            Stmt::Blit { src, dst, count, .. } =>
            {
                self.expr(src, Use::Read, false);
                self.expr(dst, Use::Written, true);
                self.expr(count, Use::Read, false);
            }
            Stmt::Fill { dst, count, value, .. } =>
            {
                self.expr(dst, Use::Written, true);
                self.expr(count, Use::Read, false);
                self.expr(value, value_use(value.kind), false);
            }
            Stmt::AddCell { dest, value, .. } =>
            {
                self.dest(dest, true);
                self.expr(value, value_use(value.kind), false);
            }
            Stmt::RemoveCell { dest, .. } => self.dest(dest, true),
            Stmt::Insert { dest, index, value, .. } =>
            {
                self.dest(dest, true);
                self.expr(index, Use::Read, false);
                self.expr(value, value_use(value.kind), false);
            }
            Stmt::Replace { dest, start, end, value, .. } =>
            {
                self.dest(dest, true);
                self.expr(start, Use::Read, false);
                self.expr(end, Use::Read, false);
                self.expr(value, Use::Read, false);
            }
            Stmt::SetChar { dest, index, value, .. } =>
            {
                self.dest(dest, false);
                self.expr(index, Use::Read, false);
                self.expr(value, Use::Read, false);
            }
            Stmt::AddText { dest, value, .. } =>
            {
                self.dest(dest, false);
                self.expr(value, Use::Read, false);
            }
        }
    }

    /// Walk the place a statement changes. The borrow checker ends each loan
    /// into a list that a write or an add or a remove makes own its cells,
    /// so the own steps of those statements are writes.
    ///
    /// # Arguments
    ///
    /// * `dest` - The place.
    /// * `write` - Whether the statement ends the loans into its lists.
    fn dest(&mut self, dest: &mut Dest, write: bool)
    {
        do_if!(let Dest::Mem(e) = dest, self.expr(e, Use::Written, write));
    }

    /// Walk the arguments of a call.
    ///
    /// # Arguments
    ///
    /// * `func` - The function called.
    /// * `args` - The arguments.
    fn args(&mut self, func: u32, args: &mut [Expr])
    {
        for (i, a) in args.iter_mut().enumerate()
        {
            let u = tern!(
                a.kind.is_slice(),
                Use::Passed(func, i as u32),
                value_use(a.kind)
            );
            self.expr(a, u, false);
        }
    }

    /// Walk an expression.
    ///
    /// # Arguments
    ///
    /// * `e` - The expression.
    /// * `u` - How its value is used.
    /// * `write` - Whether it is the address of a write that ends the loans
    ///   into the lists it makes own their cells.
    fn expr(&mut self, e: &mut Expr, u: Use, write: bool)
    {
        let kind = e.kind;
        match &mut e.node
        {
            ExprNode::Read(place) => self.read(*place, kind, u),
            ExprNode::AddrOf(place) => self.address(*place, kind),
            ExprNode::Take { addr } => self.take(addr, write),
            ExprNode::Load { addr } =>
            {
                // A list or a struct is loaded as its address.
                self.expr(addr, tern!(kind.is_aggregate(), u, Use::Read), write)
            }
            ExprNode::CellAddr { list, index, .. } =>
            {
                self.expr(list, cells_use(u), write);
                self.expr(index, Use::Read, write);
            }
            ExprNode::FieldAddr { base, .. } => self.expr(base, cells_use(u), write),
            ExprNode::MakeSlice { source, start, len, .. } =>
            {
                self.lend(source, u, write);
                self.expr(start, Use::Read, write);
                self.expr(len, Use::Read, write);
            }
            ExprNode::Own { value } =>
            {
                self.own(value, write);
                self.expr(value, Use::Read, write);
            }
            ExprNode::Convert { operand, .. } =>
            {
                let u = tern!(operand.kind.is_slice(), u, value_use(operand.kind));
                self.expr(operand, u, write);
            }
            ExprNode::Lend { operand } => self.expr(operand, u, write),
            ExprNode::PtrAdd { base, offset } =>
            {
                self.expr(base, u, write);
                self.expr(offset, Use::Read, write);
            }
            ExprNode::Call { func, args, .. } => self.args(*func, args),
            ExprNode::Compare { lhs, rhs, .. } | ExprNode::PtrSub { lhs, rhs } =>
            {
                self.expr(lhs, Use::Read, write);
                self.expr(rhs, Use::Read, write);
            }
            ExprNode::Infix { lhs, rhs, .. } | ExprNode::Logic { lhs, rhs, .. } =>
            {
                self.expr(lhs, value_use(lhs.kind), write);
                self.expr(rhs, value_use(rhs.kind), write);
            }
            ExprNode::Prefix { operand, .. } | ExprNode::Dup { value: operand, .. } =>
            {
                self.expr(operand, value_use(operand.kind), write)
            }
            ExprNode::Chars { text, index, .. } =>
            {
                self.expr(text, Use::Read, write);
                do_if!(let Some(i) = index, self.expr(i, Use::Read, write));
            }
            ExprNode::Format(pieces) =>
            {
                let values = pieces.iter_mut().filter_map(|p| match p
                {
                    FmtPiece::Value(v) => Some(v),
                    FmtPiece::Text(_) => None,
                });
                do_for!(v in values, self.expr(v, Use::Read, write));
            }
            ExprNode::List(items) =>
            {
                do_for!(i in items, self.expr(i, value_use(i.kind), write));
            }
            ExprNode::Const(_)
            | ExprNode::SizeOf(_)
            | ExprNode::BlankList
            | ExprNode::Text(_) =>
            {}
        }
    }

    /// Note a read of a slot: a slice slot takes the use.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `kind` - Its kind.
    /// * `u` - How the value is used.
    fn read(&mut self, place: Place, kind: DataKind, u: Use)
    {
        do_if!(kind.is_slice(), self.uses.push((self.spot(place), u)));
    }

    /// Note the address of a slot: a slice behind an address can be written,
    /// and a plain list behind one can own its cells where the walk cannot
    /// see it.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `kind` - The kind of the address.
    fn address(&mut self, place: Place, kind: DataKind)
    {
        let spot = self.spot(place);
        let Some(slot) = kind.pointee()
        else
        {
            return;
        };
        do_if!(slot.is_slice(), self.uses.push((spot, Use::Written)));
        do_if!(slot.is_seq(), _ = self.addressed.insert(spot));
    }

    /// Walk a move out of a place. A move out of a slot takes its value and
    /// leaves the cells of a list in place, so it reaches the slot through
    /// no pointer.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the place.
    /// * `write` - Whether it is part of the address of a write.
    fn take(&mut self, addr: &mut Expr, write: bool)
    {
        let ExprNode::AddrOf(place) = addr.node
        else
        {
            return self.expr(addr, Use::Written, write);
        };
        let slice = addr.kind.pointee().is_some_and(DataKind::is_slice);
        do_if!(slice, self.uses.push((self.spot(place), Use::Written)));
    }

    /// Note an own step on a plain list slot that sits outside a write.
    ///
    /// # Arguments
    ///
    /// * `value` - The list the step takes.
    /// * `write` - Whether the step is part of the address of a write.
    fn own(&mut self, value: &Expr, write: bool)
    {
        ret_if!(write);
        let Some(place) = plain_slot(value)
        else
        {
            return;
        };
        *self.owns.entry(self.spot(place)).or_default() += 1;
    }

    /// Walk the source of a slice. The first walk notes a slice of a plain
    /// list slot and its use; the second drops the own step of each one that
    /// reads alone, on a list [`Walk::free_lists`] lets go.
    ///
    /// # Arguments
    ///
    /// * `source` - The source.
    /// * `u` - How the slice is used; a slice of a slice is used as the slice
    ///   it is made of.
    /// * `write` - Whether it is part of the address of a write.
    fn lend(&mut self, source: &mut Expr, u: Use, write: bool)
    {
        let owned =
            tern!(let ExprNode::Own { value } = &source.node, plain_slot(value), None);
        let Some(place) = owned.filter(|_| !write)
        else
        {
            return self.expr(source, u, write);
        };
        let spot = self.spot(place);
        let Some(freed) = &self.freed
        else
        {
            self.lends.push((spot, u));
            return self.expr(source, u, write);
        };
        ret_if!(!freed.contains(&spot) || self.is_written(u));
        let node = std::mem::replace(&mut source.node, ExprNode::BlankList);
        extract!(ExprNode::Own { value } = node, "the source is an own step");
        *source = *value;
    }
}
