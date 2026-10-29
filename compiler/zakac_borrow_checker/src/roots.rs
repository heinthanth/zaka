use std::collections::BTreeSet;

use bon::Builder;
use zakac_ir_builder::ir::Dest;
use zakac_ir_builder::ir::Expr;
use zakac_ir_builder::ir::ExprNode;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
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
        ExprNode::Lend { operand }
        | ExprNode::Prefix { operand, .. }
        | ExprNode::Convert { operand, .. }
        | ExprNode::PtrAdd { base: operand, .. } => deeper(operand),
        ExprNode::Load { addr } => load_roots(addr, e.kind, state, ctx),
        ExprNode::CellAddr { list, index, first, .. } =>
        {
            let n = cell_number(index, *first);
            list_roots(list, state, ctx).iter().map(|r| r.cell(n)).collect()
        }
        ExprNode::List(cells) => cells.iter().flat_map(deeper).collect(),
        ExprNode::Call { func, args, .. } => call_roots(*func, args, state, ctx),
        _ => Roots::new(),
    }
}

/// The roots of a slot read. The slot decides: an owner slot read as the
/// address of what it holds (`ပ ၏ တန်ဖိုး ၏ လိပ်စာ`) still points into
/// its block, though the read is of a borrow kind.
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
    let owner = ctx.kind_of(place).unwrap_or(kind).is_owner_ptr();
    ret_if!(owner, Roots::from([Root::slot(place).block()]));
    state.ptr(place).map(|info| info.roots.clone()).unwrap_or_default()
}

/// The roots of a load. A borrow read out of a cell of a list slot points
/// where the borrows of the list point, so it gets the roots the state
/// holds for the whole list; another load gets the block of each place
/// the address names. The cell decides, as a slot does: a cell that owns
/// a block, read as the address of what it holds, points into its block.
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
    let borrowed =
        kind.holds_borrow() && !cell_kind(addr).is_some_and(DataKind::is_owner_ptr);
    let list = borrowed.then(|| list_slot(addr)).flatten();
    ret_if!(let Some(info) = list.and_then(|place| state.ptr(place)), info.roots.clone());
    roots_of(addr, state, ctx).iter().map(Root::block).collect()
}

/// The kind of the cell a cell address names.
///
/// # Arguments
///
/// * `addr` - The address.
///
/// # Returns
///
/// The cell kind, or `None` for an address that is no cell of a list.
fn cell_kind(addr: &Expr) -> Option<DataKind>
{
    let ExprNode::CellAddr { list, .. } = &addr.node
    else
    {
        return None;
    };
    list.kind.as_list().map(|l| l.cell)
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
/// The slot of a list read from a slot, the roots of the address of a
/// list read from mem, the roots of a pointer to a list.
fn list_roots(list: &Expr, state: &State, ctx: &RootCtx) -> Roots
{
    ret_if!(!list.kind.is_list(), roots_of(list, state, ctx));
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

/// The steps from a slot of `kind` to the blocks its owners hold: one
/// step into each cell of each level of a list, then the block.
///
/// # Arguments
///
/// * `kind` - The kind of the slot.
///
/// # Returns
///
/// The steps; a block step alone for a kind that is no list.
fn owner_steps(kind: DataKind) -> Vec<Seg>
{
    let Some(list) = kind.as_list()
    else
    {
        return vec![Seg::Block];
    };
    [vec![Seg::Cell(None)], owner_steps(list.cell)].concat()
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
/// can point at.
pub(crate) fn freed_paths(dest: &Dest, state: &State, ctx: &RootCtx) -> Roots
{
    match dest
    {
        Dest::Slot(place) => Roots::from([Root::slot(*place).block()]),
        Dest::Mem(addr) => roots_of(addr, state, ctx).iter().map(Root::block).collect(),
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
/// moved out, through each level of nested lists; none for another
/// expression.
pub(crate) fn take_paths(take: &Expr, state: &State, ctx: &RootCtx) -> Roots
{
    let ExprNode::Take { addr } = &take.node
    else
    {
        return Roots::new();
    };
    let steps = owner_steps(take.kind);
    roots_of(addr, state, ctx).iter().map(|r| r.then(&steps)).collect()
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
        ret_if!(!value.kind.holds_borrow());
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
    fn global_roots_union_top_level_sets()
    {
        let module = lower(
            "ကိန်းရှင် က သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို က တွင် သိမ်းထားမည်။
မင်မိုရီ ကိန်းပြည့် ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဂ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် က ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
ကိန်းရှင် ဃ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ဂ + ၁ ဖြစ်ပါစေ။
ဂ ကို ခ ၏ တန်ဖိုးအငှား ဟု ထားပါ။
",
        );
        let roots = global_roots(&module, &[]);
        let a = Root::slot(Place::Global(0)).block();
        let b = Root::slot(Place::Global(1)).block();
        assert!(roots[0].is_empty() && roots[1].is_empty());
        assert_eq!(roots[2], Roots::from([a.clone(), b.clone()]));
        assert_eq!(roots[3], Roots::from([a, b]));
    }

    #[test]
    fn a_borrow_read_from_a_cell_points_where_the_list_points()
    {
        let module = lower(
            "ကိန်းရှင် ပ သည် ကိန်းပြည့် လိပ်စာ ဖြစ်ပါစေ။
ကိန်းရှင် တန်း သည် ငှားထားသော အကွက် ၂ ခုရှိသော ကိန်းပြည့် လိပ်စာ စာရင်း ဖြစ်၍ တန်ဖိုးသည် (ပ ၏ တန်ဖိုးအငှား)၊ (ပ ၏ \
             တန်ဖိုးအငှား) တို့ပါသော စာရင်း ဖြစ်ပါစေ။
ကိန်းရှင် ဂ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် တန်း ၏ အကွက် ၁ ဖြစ်ပါစေ။
",
        );
        let roots = global_roots(&module, &[]);
        let owner = Roots::from([Root::slot(Place::Global(0)).block()]);
        assert_eq!(roots[1], owner);
        assert_eq!(roots[2], owner);
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

    #[test]
    fn load_through_an_owner_adds_a_block_step()
    {
        let module = lower(
            "ကိန်းရှင် ခ သည် ကိန်းပြည့် လိပ်စာ ၏ လိပ်စာ ဖြစ်ပါစေ။
မင်မိုရီ ကိန်းပြည့် လိပ်စာ ၁ ခုစာ နေရာယူ၍ လိပ်စာကို ခ တွင် သိမ်းထားမည်။
ကိန်းရှင် ဂ သည် ငှားထားသော ကိန်းပြည့် လိပ်စာ ဖြစ်၍ တန်ဖိုးသည် ခ ၏ တန်ဖိုး ၏ တန်ဖိုးအငှား ဖြစ်ပါစေ။
",
        );
        let roots = global_roots(&module, &[]);
        assert_eq!(
            roots[1],
            Roots::from([Root::slot(Place::Global(0)).block().block()])
        );
    }
}
