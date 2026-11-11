use std::collections::BTreeSet;

use bon::Builder;
use zakac_ir_builder::ir::Access;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::SliceFrom;
use zakac_ir_builder::ir::Slot;
use zakac_ir_builder::ir::Stmt;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;

use crate::digest::FnDigest;
use crate::digest::MAX_PATH;
use crate::digest::Root;
use crate::digest::Seg;
use crate::interval::const_int;
use crate::state::Interval;
use crate::state::PtrInfo;
use crate::state::SlotState;
use crate::state::State;
use crate::state::holds_address;
use crate::values::value_callees;
use crate::walk::each_stmt;

/// The places a pointer can point into.
pub(crate) type Roots = BTreeSet<Root>;

/// What [`roots_of`] reads besides the state.
#[derive(Debug, Clone, Copy, Builder)]
pub(crate) struct RootCtx<'m>
{
    /// The module.
    pub(crate) module: &'m Module,
    /// The local slots of the routine; none for the main program.
    pub(crate) locals: &'m [Slot],
    /// The roots top-level code gives each global, see [`global_roots`].
    pub(crate) global_roots: &'m [Roots],
    /// The digest of each function so far; a call to a function past the
    /// end can return a borrow of each argument.
    pub(crate) digests: &'m [FnDigest],
}

impl RootCtx<'_>
{
    /// The kind of a slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for a slot out of range.
    pub(crate) fn kind_of(&self, place: Place) -> Option<DataKind>
    {
        let slot = match place
        {
            Place::Global(n) => self.module.globals.get(n as usize),
            Place::Local(n) => self.locals.get(n as usize),
        };
        slot.map(|s| s.kind)
    }

    /// Whether a slot is a constant, whose text is static, so a read of it
    /// is a duplicate and never a move.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// `true` for a `ကိန်းသေ`.
    pub(crate) fn is_const(&self, place: Place) -> bool
    {
        let slot = match place
        {
            Place::Global(n) => self.module.globals.get(n as usize),
            Place::Local(n) => self.locals.get(n as usize),
        };
        slot.is_some_and(|s| s.access == Access::Const)
    }
}

/// The places the pointer an expression gives can point into.
///
/// # Arguments
///
/// * `e` - The expression.
/// * `state` - The facts before it runs.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The roots; none for a raw address and for a value that is no pointer.
pub(crate) fn roots_of(e: &Expr, state: &State, ctx: &RootCtx) -> Roots
{
    ret_if!(e.kind.is_raw_ptr(), Roots::new());
    let deeper = |inner: &Expr| roots_of(inner, state, ctx);
    match &e.node
    {
        ExprNode::AddrOf(place) => Roots::from([Root::slot(*place)]),
        ExprNode::Read(place) => read_roots(*place, e.kind, state, ctx),
        // A pointer made from an object holds a count of it, not a loan.
        ExprNode::Convert { operand, .. } if operand.kind.as_object().is_some() =>
        {
            Roots::new()
        }
        // A duplicate of text or a list holds a count of its own, so no free
        // through another holder reaches it; a box duplicated for a capture
        // shares the one box, and what the box holds can be freed.
        ExprNode::Dup { value, .. } if e.kind.is_box() || e.kind.is_ptr() =>
        {
            deeper(value)
        }
        ExprNode::Lend { operand }
        | ExprNode::Own { value: operand, .. }
        | ExprNode::Prefix { operand, .. }
        | ExprNode::Convert { operand, .. }
        | ExprNode::PtrAdd { base: operand, .. } => deeper(operand),
        ExprNode::Load { addr } => load_roots(addr, e.kind, state, ctx),
        // The cells of a plain list sit in a block the value points at.
        ExprNode::CellAddr { list, .. } if list.kind.is_seq() => deeper(list),
        ExprNode::MakeSlice { source, from: SliceFrom::List(_), .. } =>
        {
            list_roots(source, state, ctx).iter().map(|r| r.cell(None)).collect()
        }
        ExprNode::MakeSlice { source, .. } => deeper(source),
        ExprNode::CellAddr { list, index, first, .. } =>
        {
            let n = cell_number(index, *first);
            list_roots(list, state, ctx).iter().map(|r| r.cell(n)).collect()
        }
        ExprNode::FieldAddr { base, field, .. } =>
        {
            list_roots(base, state, ctx).iter().map(|r| r.cell(Some(*field))).collect()
        }
        ExprNode::List(cells) => cells.iter().flat_map(deeper).collect(),
        ExprNode::Call { func, args, .. } => call_roots(*func, args, state, ctx),
        ExprNode::CallValue { callee, args } =>
        {
            let funcs = value_callees(ctx.module, callee.kind);
            ret_if!(
                funcs.is_empty(),
                args.iter().flat_map(|a| roots_of(a, state, ctx)).collect()
            );
            funcs.iter().flat_map(|f| call_roots(*f, args, state, ctx)).collect()
        }
        _ => Roots::new(),
    }
}

/// The roots of a slot read. The slot decides: an owner slot read as the
/// address of what it holds (`ပ ၏ တန်ဖိုး ၏ လိပ်စာ`) still points into
/// its block, though the read is of a borrow kind; so does a text slot,
/// whose u8s sit in the block it owns.
///
/// # Arguments
///
/// * `place` - The slot.
/// * `kind` - The kind of the read.
/// * `state` - The facts.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The block of an owner, the roots the state holds for a borrow.
fn read_roots(place: Place, kind: DataKind, state: &State, ctx: &RootCtx) -> Roots
{
    let held = ctx.kind_of(place).unwrap_or(kind);
    let roots = state.ptr(place).map(|info| info.roots.clone()).unwrap_or_default();
    // A pointer that points into a known place reads as that place; one into
    // none, such as a new block, is a block of its own.
    ret_if!(held.is_owner_ptr() && !roots.is_empty(), roots);
    let owner = held.is_owner_ptr() || held.is_text() || held.is_seq();
    ret_if!(owner, Roots::from([Root::slot(place).block()]));
    roots
}

/// The roots of a load. A pointer read out of a cell of a list slot points
/// where the pointers stored in the list point, so it gets the roots the
/// state holds for the whole list. A load with no such roots, such as a
/// cell that a new block was stored into, gets the block of each place the
/// address names.
///
/// # Arguments
///
/// * `addr` - The address loaded from.
/// * `kind` - The kind of the value loaded.
/// * `state` - The facts.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The roots.
fn load_roots(addr: &Expr, kind: DataKind, state: &State, ctx: &RootCtx) -> Roots
{
    let list = holds_address(kind).then(|| list_slot(addr)).flatten();
    // Roots that lead back into the list alone, as a parameter starts with,
    // tell less than the cell the address names.
    let stored =
        |place: Place, info: &PtrInfo| info.roots.iter().any(|r| r.place != place);
    let held = list.and_then(|place| state.ptr(place).filter(|i| stored(place, i)));
    ret_if!(let Some(info) = held, info.roots.clone());
    roots_of(addr, state, ctx).iter().map(Root::block).collect()
}

/// The list slot a cell address reads, through the cells of nested lists.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The slot, or `None` for an address that is no cell of a list slot.
pub(crate) fn list_slot(addr: &Expr) -> Option<Place>
{
    let ExprNode::CellAddr { list, .. } = &addr.node
    else
    {
        return None;
    };
    ret_if!(!list.kind.is_list(), None);
    match &list.node
    {
        ExprNode::Read(place) => Some(*place),
        ExprNode::Load { addr } => list_slot(addr),
        _ => None,
    }
}

/// Where the list of a cell address lives.
///
/// # Arguments
///
/// * `list` - The list, or a pointer to it.
/// * `state` - The facts.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The slot of a list or struct read from a slot, the roots of the address
/// of one read from mem, the roots of a pointer to a list.
fn list_roots(list: &Expr, state: &State, ctx: &RootCtx) -> Roots
{
    let view = !list.kind.is_aggregate();
    ret_if!(view, roots_of(list, state, ctx));
    match &list.node
    {
        ExprNode::Read(place) => Roots::from([Root::slot(*place)]),
        ExprNode::Load { addr } => roots_of(addr, state, ctx),
        _ => Roots::new(),
    }
}

/// The cell a constant cell number names.
///
/// # Arguments
///
/// * `index` - The cell number.
/// * `first` - The number of the first cell.
///
/// # Returns
///
/// The cell from 0, or `None` when the number is not a constant.
fn cell_number(index: &Expr, first: u8) -> Option<u32>
{
    let n = const_int(index)?.checked_sub(i128::from(first))?;
    u32::try_from(n).ok()
}

/// The roots of a call result: the paths its digest gives, each moved
/// over to the arguments of this call.
///
/// # Arguments
///
/// * `func` - The function number.
/// * `args` - The arguments.
/// * `state` - The facts.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The roots; those of each argument when the function has no digest.
fn call_roots(func: u32, args: &[Expr], state: &State, ctx: &RootCtx) -> Roots
{
    let Some(digest) = ctx.digests.get(func as usize)
    else
    {
        return args.iter().flat_map(|a| roots_of(a, state, ctx)).collect();
    };
    let bases: Vec<Roots> = args.iter().map(|a| value_roots(a, state, ctx)).collect();
    digest.result_roots.iter().flat_map(|r| translate(r, &bases)).collect()
}

/// The places the value an argument hands over can point into: the
/// block a move gives on, or the roots of each other value.
///
/// # Arguments
///
/// * `arg` - The argument.
/// * `state` - The facts.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The roots.
pub(crate) fn value_roots(arg: &Expr, state: &State, ctx: &RootCtx) -> Roots
{
    ret_if!(matches!(arg.node, ExprNode::Take { .. }), take_paths(arg, state, ctx));
    roots_of(arg, state, ctx)
}

/// The places of the caller a path of the callee names. A parameter
/// starts out pointing where its argument points, so a path through the
/// block of a parameter, or through the cells of a list parameter at
/// whatever depth of nested lists, goes on from each root of the
/// argument; a global path names the same place in each routine. The
/// roots of a list argument are those of each of its cells, so the cell
/// numbers of the path are not kept.
///
/// # Arguments
///
/// * `root` - The path, from a parameter or a global of the callee.
/// * `args` - The roots of each argument, see [`value_roots`].
///
/// # Returns
///
/// The paths in the caller.
pub(crate) fn translate(root: &Root, args: &[Roots]) -> Roots
{
    let Place::Local(n) = root.place
    else
    {
        return Roots::from([root.clone()]);
    };
    let Some(base) = args.get(n as usize)
    else
    {
        return Roots::new();
    };
    let rest = match past_cells(&root.path)
    {
        [Seg::Block, rest @ ..] => rest,
        _ => &[],
    };
    base.iter().map(|b| b.then(rest)).collect()
}

/// The steps of a path after the cell steps it starts with: from a list
/// slot, at whatever depth of nested lists, on from the cell.
///
/// # Arguments
///
/// * `path` - The steps.
///
/// # Returns
///
/// The steps after the leading cell steps.
pub(crate) fn past_cells(path: &[Seg]) -> &[Seg]
{
    let cells = path.iter().take_while(|s| matches!(s, Seg::Cell(_))).count();
    &path[cells..]
}

/// The blocks the cells of a list slot hold, for a list duplicated whole
/// into another slot: the new list points where these cells do.
///
/// # Arguments
///
/// * `place` - The list slot.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The path from the slot to the block of each cell, at each depth.
pub(crate) fn cell_blocks(place: Place, kind: DataKind) -> Roots
{
    owner_steps(kind).into_iter().map(|steps| Root::slot(place).then(&steps)).collect()
}

/// The paths from a slot of `kind` to the blocks its owners hold: one
/// step into each cell of each level of a list, or into each field of a
/// struct that holds an owner, then the block.
///
/// # Arguments
///
/// * `kind` - The kind of the slot.
///
/// # Returns
///
/// The paths; a block step alone for a kind that is neither a list nor a
/// struct.
fn owner_steps(kind: DataKind) -> Vec<Vec<Seg>>
{
    let into = |seg: Seg, inner: DataKind| {
        owner_steps(inner).into_iter().map(move |rest| [vec![seg], rest].concat())
    };
    ret_if!(let Some(list) = kind.as_list(), into(Seg::Cell(None), list.cell).collect());
    let Some(fields) = kind.as_struct().map(|s| s.fields())
    else
    {
        return vec![vec![Seg::Block]];
    };
    let moved = fields.iter().enumerate().filter(|(_, (_, k))| k.holds_owner_ptr());
    moved.flat_map(|(at, (_, k))| into(Seg::Cell(u32::try_from(at).ok()), *k)).collect()
}

/// The paths a free or a resize of `dest` gives back.
///
/// # Arguments
///
/// * `dest` - Where the owner lives.
/// * `state` - The facts before the statement.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The block of the owner slot, or the blocks of the owners the address
/// can point at, with the blocks the pointers stored there point at.
pub(crate) fn freed_paths(dest: &Dest, state: &State, ctx: &RootCtx) -> Roots
{
    match dest
    {
        // The block of the slot, and for a global each block a duplicate in it
        // points at, such as a parameter's block that the global took: a free
        // reaches it too, and the callers see it in the digest.
        Dest::Slot(place) =>
        {
            let global = matches!(place, Place::Global(_));
            let held = state.ptr(*place).filter(|_| global).map(|i| i.roots.clone());
            let held = held.unwrap_or_default();
            let blocks =
                held.into_iter().filter(|r| r.path.last() == Some(&Seg::Block));
            blocks.chain([Root::slot(*place).block()]).collect()
        }
        Dest::Mem(addr) =>
        {
            let held = addr.kind.pointee().unwrap_or(addr.kind);
            let stored = load_roots(addr, held, state, ctx);
            roots_of(addr, state, ctx).iter().map(Root::block).chain(stored).collect()
        }
    }
}

/// The one block an allocation at `dest` fills, when the destination
/// names one block for certain: one path, with each cell known and short
/// of the cap, since a capped path stands for deeper ones too.
///
/// # Arguments
///
/// * `dest` - Where the owner lives.
/// * `state` - The facts before the statement.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The path, or `None` when the destination can name several blocks.
pub(crate) fn filled_path(dest: &Dest, state: &State, ctx: &RootCtx) -> Option<Root>
{
    let paths = freed_paths(dest, state, ctx);
    ret_if!(paths.len() != 1, None);
    let short = |r: &&Root| r.path.len() < MAX_PATH;
    paths.first().filter(short).filter(|r| known_cells(r)).cloned()
}

/// Whether each cell step of a path names its cell.
///
/// # Arguments
///
/// * `root` - The path.
///
/// # Returns
///
/// `false` when a step stands for each cell of a list.
pub(crate) fn known_cells(root: &Root) -> bool { !root.path.contains(&Seg::Cell(None)) }

/// The paths a move out of a place leaves without their block.
///
/// # Arguments
///
/// * `take` - The [`ExprNode::Take`] expression.
/// * `state` - The facts before it runs.
/// * `ctx` - The module and the digests.
///
/// # Returns
///
/// The block of the owner moved out, or the blocks of each cell of a list
/// or each owner field of a struct moved out, through each level of nested
/// lists and structs; none for another expression.
pub(crate) fn take_paths(take: &Expr, state: &State, ctx: &RootCtx) -> Roots
{
    let ExprNode::Take { addr } = &take.node
    else
    {
        return Roots::new();
    };
    let paths = owner_steps(take.kind);
    let roots = roots_of(addr, state, ctx);
    roots.iter().flat_map(|r| paths.iter().map(|steps| r.then(steps))).collect()
}

/// The roots top-level code gives each global: the union over each set of
/// the global, to a fixpoint, since a set can read another global. A
/// function that reads a global borrow borrows from each of them.
///
/// # Arguments
///
/// * `module` - The module.
/// * `digests` - The digest of each function, for the roots of a call result.
///
/// # Returns
///
/// The roots of each global, in module order; none for a global with no
/// borrow.
pub(crate) fn global_roots(module: &Module, digests: &[FnDigest]) -> Vec<Roots>
{
    let mut found = vec![Roots::new(); module.globals.len()];
    loop
    {
        let next = global_round(module, &found, digests);
        ret_if!(next == found, found);
        found = next;
    }
}

/// One round of [`global_roots`].
///
/// # Arguments
///
/// * `module` - The module.
/// * `found` - The roots of each global so far.
/// * `digests` - The digest of each function.
///
/// # Returns
///
/// The roots with those of each set added.
fn global_round(module: &Module, found: &[Roots], digests: &[FnDigest]) -> Vec<Roots>
{
    let seed = |(slot, roots): (&Slot, &Roots)| {
        let info = PtrInfo::unknown(roots.clone());
        SlotState::of_kind(slot.kind, info, Interval::TOP)
    };
    let globals = module.globals.iter().zip(found).map(seed).collect();
    let state = State::builder().globals(globals).build();
    let ctx = RootCtx::builder()
        .module(module)
        .locals(&[])
        .global_roots(found)
        .digests(digests)
        .build();
    let mut next = found.to_vec();
    each_stmt(&module.main, &mut |s| {
        let Some((g, value)) = global_write(s)
        else
        {
            return;
        };
        ret_if!(!holds_address(value.kind));
        next[g as usize].extend(roots_of(value, &state, &ctx));
    });
    next
}

/// The global a statement writes a value into, and the value: a set of
/// the global, or a store into a cell of a global list, which adds to the
/// roots of the list as a set adds to those of a slot.
///
/// # Arguments
///
/// * `s` - The statement.
///
/// # Returns
///
/// The global number and the value, or `None` for each other statement.
fn global_write(s: &Stmt) -> Option<(u32, &Expr)>
{
    let (place, value) = match s
    {
        Stmt::Set { place, value, .. } => (*place, value),
        Stmt::Store { addr, value, .. } => (list_slot(addr)?, value),
        _ => return None,
    };
    tern!(let Place::Global(g) = place, Some((g, value)), None)
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::testing::lower;

    #[test]
    fn global_roots_follow_an_initializer_through_another()
    {
        let module = lower(
            "ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် က ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် ဂ ဖြစ်ပါစေ။
ပရိုဂရမ်သည် အောက်ပါတို့ကို လုပ်ဆောင်မည်။
ပရိုဂရမ် ပြီးဆုံးပါပြီး။
",
        );
        let roots = global_roots(&module, &[]);
        let a = Root::slot(Place::Global(0)).block();
        assert!(roots[0].is_empty() && roots[1].is_empty());
        assert_eq!(roots[2], Roots::from([a.clone()]));
        // A global read before its roots are known reads as a block of its
        // own, and the union over the rounds keeps that block.
        let c = Root::slot(Place::Global(2)).block();
        assert_eq!(roots[3], Roots::from([a, c]));
    }

    #[test]
    fn a_pointer_read_from_a_cell_points_where_the_list_points()
    {
        let module = lower(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်ပါစေ။
ကိန်းရှင် တန်း သည် အကွက် ၂ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း တစ်ခု ဖြစ်၍ တန်ဖိုးသည် (ပ)၊ (ပ) တို့ပါသော စာရင်း \
             ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ကိန်းပြည့် လိပ်စာ တစ်ခု ဖြစ်၍ တန်ဖိုးသည် တန်း ၏ အကွက် ၁ ဖြစ်ပါစေ။
",
        );
        let roots = global_roots(&module, &[]);
        let owner = Roots::from([Root::slot(Place::Global(0)).block()]);
        assert_eq!(roots[1], owner);
        // The fixpoint can keep the cell block the first round read, too.
        assert!(roots[2].is_superset(&owner), "{:?}", roots[2]);
    }

    #[test]
    fn a_callee_path_goes_on_from_each_argument_root()
    {
        let (p, q) = (Root::slot(Place::Global(0)), Root::slot(Place::Global(1)));
        let args = [Roots::from([p.clone()]), Roots::from([q.block()])];
        let through = Root::slot(Place::Local(0)).block().block();
        assert_eq!(translate(&through, &args), Roots::from([p.block()]));
        let cells = Root::slot(Place::Local(1)).cell(Some(2)).block().block();
        assert_eq!(translate(&cells, &args), Roots::from([q.block().block()]));
        let global = Root::slot(Place::Global(3)).block();
        assert_eq!(translate(&global, &args), Roots::from([global.clone()]));
        assert!(translate(&Root::slot(Place::Local(5)).block(), &args).is_empty());
    }
}
