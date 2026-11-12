use std::collections::BTreeMap;
use std::collections::BTreeSet;

use bon::Builder;
use imbl::Vector;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Slot;
use zakac_rust_sugars::do_for;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::tern;
use zakac_semcheck::kind::DataKind;
use zakac_semcheck::kind::IntKind;

use crate::digest::Root;
use crate::roots::Roots;
use crate::routine::Routine;

/// What an owner can hold at a point: a set of four facts, joined with a
/// bit or, and a mark for a block a call can have given back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct OwnerState(u8);

impl OwnerState
{
    /// null.
    pub(crate) const NULL: OwnerState = OwnerState(1);
    /// A block.
    pub(crate) const HELD: OwnerState = OwnerState(2);
    /// A fresh allocation not compared with null, so a block or null.
    pub(crate) const FRESH: OwnerState = OwnerState(4);
    /// Nothing: the value moved out and left null.
    pub(crate) const MOVED: OwnerState = OwnerState(8);
    /// A mark, not a fact: a call can have freed, resized or moved the
    /// block through an address of the slot, so the slot is not reported
    /// as a leak until the routine writes it again.
    pub(crate) const GIVEN: OwnerState = OwnerState(16);
    /// A mark, not a fact: the value is a duplicate of a fresh allocation
    /// not compared with null, so `--mem-strict` warns at a read through
    /// it, as through the allocation itself; a test against null ends it.
    pub(crate) const UNTESTED: OwnerState = OwnerState(32);
    /// A block or null, with nothing known.
    pub(crate) const UNKNOWN: OwnerState = OwnerState(1 | 2);
    /// No fact: a point no path reaches.
    pub(crate) const NONE: OwnerState = OwnerState(0);

    /// The facts of either state.
    ///
    /// # Arguments
    ///
    /// * `other` - The other state.
    ///
    /// # Returns
    ///
    /// The union of the facts.
    pub(crate) const fn join(self, other: OwnerState) -> OwnerState
    {
        OwnerState(self.0 | other.0)
    }

    /// Whether the state shares a fact with `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The other state.
    ///
    /// # Returns
    ///
    /// `true` when a fact is in both.
    const fn shares(self, other: OwnerState) -> bool { self.0 & other.0 != 0 }

    /// The state without the facts of `other`.
    ///
    /// # Arguments
    ///
    /// * `other` - The facts to drop.
    ///
    /// # Returns
    ///
    /// The smaller state.
    pub(crate) const fn without(self, other: OwnerState) -> OwnerState
    {
        OwnerState(self.0 & !other.0)
    }

    /// Whether the owner moved out on some path.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::MOVED`].
    pub(crate) const fn shares_moved(self) -> bool { self.shares(OwnerState::MOVED) }

    /// Whether a call can have given the block back on some path.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::GIVEN`].
    pub(crate) const fn shares_given(self) -> bool { self.shares(OwnerState::GIVEN) }

    /// Whether the owner is null on each path.
    ///
    /// # Returns
    ///
    /// `true` for [`OwnerState::NULL`] alone.
    pub(crate) fn is_null(self) -> bool { self == OwnerState::NULL }

    /// Whether the owner moved out on each path.
    ///
    /// # Returns
    ///
    /// `true` for [`OwnerState::MOVED`] alone.
    pub(crate) fn is_moved(self) -> bool { self == OwnerState::MOVED }

    /// Whether the owner can hold a block.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::HELD`] or [`OwnerState::FRESH`].
    pub(crate) const fn can_hold(self) -> bool
    {
        self.shares(OwnerState(OwnerState::HELD.0 | OwnerState::FRESH.0))
    }

    /// Whether the owner holds a block on each path.
    ///
    /// # Returns
    ///
    /// `true` when each fact is [`OwnerState::HELD`] or
    /// [`OwnerState::FRESH`].
    pub(crate) const fn must_hold(self) -> bool
    {
        let rest = self.without(OwnerState(OwnerState::HELD.0 | OwnerState::FRESH.0));
        self.can_hold() && rest.0 == OwnerState::NONE.0
    }

    /// Whether the owner can be null though it was never compared with
    /// it, or is null on some path.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::NULL`] or [`OwnerState::FRESH`].
    #[cfg(test)]
    pub(crate) const fn possible_null(self) -> bool
    {
        self.shares(OwnerState(OwnerState::NULL.0 | OwnerState::FRESH.0))
    }

    /// Whether the owner can hold an allocation not compared with null
    /// since it was made.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::FRESH`].
    pub(crate) const fn shares_fresh(self) -> bool { self.shares(OwnerState::FRESH) }

    /// Whether the owner can hold an allocation, or a duplicate of one, not
    /// compared with null since it was made.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::FRESH`] or [`OwnerState::UNTESTED`].
    pub(crate) const fn shares_untested(self) -> bool
    {
        self.shares(OwnerState(OwnerState::FRESH.0 | OwnerState::UNTESTED.0))
    }

    /// The state a read of the owner sees: moved out reads as null, and
    /// the mark of the slot remains with the slot.
    ///
    /// # Returns
    ///
    /// The state with [`OwnerState::MOVED`] turned into
    /// [`OwnerState::NULL`], without [`OwnerState::GIVEN`].
    pub(crate) const fn as_read(self) -> OwnerState
    {
        let rest = self.without(OwnerState(OwnerState::MOVED.0 | OwnerState::GIVEN.0));
        tern!(self.shares_moved(), rest.join(OwnerState::NULL), rest)
    }

    /// The state on one edge of a test against null.
    ///
    /// # Arguments
    ///
    /// * `null` - `true` on the edge where the owner is null.
    ///
    /// # Returns
    ///
    /// On the null edge: null for each fact that allows it, moved out
    /// kept; on the other edge: a block for each fact that allows one.
    /// [`OwnerState::NONE`] when the edge cannot run. The test tells
    /// whether a call gave the block back, so the mark goes.
    pub(crate) const fn narrow(self, null: bool) -> OwnerState
    {
        let fresh = self.shares(OwnerState::FRESH);
        let kept =
            tern!(null, OwnerState::NULL.join(OwnerState::MOVED), OwnerState::HELD);
        let from_fresh = tern!(null, OwnerState::NULL, OwnerState::HELD);
        let base = OwnerState(self.0 & kept.0);
        tern!(fresh, base.join(from_fresh), base)
    }
}

/// The integers a value can hold, from `lo` to `hi`, both included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Interval
{
    /// The least value.
    pub(crate) lo: i128,
    /// The greatest value.
    pub(crate) hi: i128,
}

impl Interval
{
    /// Each integer.
    pub(crate) const TOP: Interval = Interval { lo: i128::MIN, hi: i128::MAX };

    /// Whether each end is a known value, not an end of [`Interval::TOP`].
    ///
    /// # Returns
    ///
    /// `true` when both ends are finite.
    pub(crate) const fn is_bounded(self) -> bool
    {
        self.lo != i128::MIN && self.hi != i128::MAX
    }

    /// The interval of one value.
    ///
    /// # Arguments
    ///
    /// * `n` - The value.
    ///
    /// # Returns
    ///
    /// `[n, n]`.
    pub(crate) const fn point(n: i128) -> Interval { Interval { lo: n, hi: n } }

    /// The values an integer kind holds.
    ///
    /// # Arguments
    ///
    /// * `kind` - The sign and width.
    /// * `platform` - The bit count of the platform-sized integer kinds.
    ///
    /// # Returns
    ///
    /// The range.
    pub(crate) fn of_int_kind(kind: IntKind, platform: u16) -> Interval
    {
        let shift = 128 - u32::from(kind.bits(platform)).clamp(1, 64);
        ret_if!(
            kind.signed,
            Interval { lo: i128::MIN >> shift, hi: i128::MAX >> shift }
        );
        Interval { lo: 0, hi: (u128::MAX >> shift) as i128 }
    }

    /// This interval when a value of `kind` holds each of its values, or
    /// [`Interval::TOP`] when one of them leaves the range of the kind,
    /// since the run time wraps it.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of the value.
    /// * `platform` - The bit count of the platform-sized integer kinds.
    ///
    /// # Returns
    ///
    /// The interval; unchanged for a kind that is no integer.
    pub(crate) fn fit(self, kind: DataKind, platform: u16) -> Interval
    {
        let Some(int) = kind.int_kind()
        else
        {
            return self;
        };
        let range = Interval::of_int_kind(int, platform);
        tern!(range.lo <= self.lo && self.hi <= range.hi, self, Interval::TOP)
    }

    /// The interval from two bounds, or [`Interval::TOP`] when one of them
    /// overflowed.
    ///
    /// # Arguments
    ///
    /// * `lo` - The least value, if it fits.
    /// * `hi` - The greatest value, if it fits.
    ///
    /// # Returns
    ///
    /// The interval.
    fn checked(lo: Option<i128>, hi: Option<i128>) -> Interval
    {
        let (Some(lo), Some(hi)) = (lo, hi)
        else
        {
            return Interval::TOP;
        };
        Interval { lo, hi }
    }

    /// The least interval that holds both.
    ///
    /// # Arguments
    ///
    /// * `other` - The other interval.
    ///
    /// # Returns
    ///
    /// The hull.
    pub(crate) fn hull(self, other: Interval) -> Interval
    {
        Interval { lo: self.lo.min(other.lo), hi: self.hi.max(other.hi) }
    }

    /// The sums of a value of each interval.
    ///
    /// # Arguments
    ///
    /// * `other` - The other interval.
    ///
    /// # Returns
    ///
    /// The interval, [`Interval::TOP`] when a bound overflows.
    pub(crate) fn plus(self, other: Interval) -> Interval
    {
        ret_if!(self == Interval::TOP || other == Interval::TOP, Interval::TOP);
        Interval::checked(self.lo.checked_add(other.lo), self.hi.checked_add(other.hi))
    }

    /// The differences of a value of each interval.
    ///
    /// # Arguments
    ///
    /// * `other` - The interval subtracted.
    ///
    /// # Returns
    ///
    /// The interval, [`Interval::TOP`] when a bound overflows.
    pub(crate) fn minus(self, other: Interval) -> Interval
    {
        ret_if!(self == Interval::TOP || other == Interval::TOP, Interval::TOP);
        Interval::checked(self.lo.checked_sub(other.hi), self.hi.checked_sub(other.lo))
    }

    /// The products of a value of each interval.
    ///
    /// # Arguments
    ///
    /// * `other` - The other interval.
    ///
    /// # Returns
    ///
    /// The interval, [`Interval::TOP`] when a product overflows.
    pub(crate) fn times(self, other: Interval) -> Interval
    {
        let ends = [
            self.lo.checked_mul(other.lo),
            self.lo.checked_mul(other.hi),
            self.hi.checked_mul(other.lo),
            self.hi.checked_mul(other.hi),
        ];
        let Some(ends) = ends.into_iter().collect::<Option<Vec<i128>>>()
        else
        {
            return Interval::TOP;
        };
        let lo = ends.iter().min().copied();
        Interval::checked(lo, ends.iter().max().copied())
    }

    /// The negated values.
    ///
    /// # Returns
    ///
    /// The interval, [`Interval::TOP`] when a bound overflows.
    pub(crate) fn negate(self) -> Interval
    {
        Interval::checked(self.hi.checked_neg(), self.lo.checked_neg())
    }

    /// This interval with each end that moved since `older` sent to its
    /// limit, so a loop reaches a fixed point in two steps for each end. An
    /// end that held is kept: a counter that grows from 0 keeps 0 as its
    /// least value.
    ///
    /// # Arguments
    ///
    /// * `older` - The interval at the last visit.
    ///
    /// # Returns
    ///
    /// The widened interval.
    fn widen(self, older: Interval) -> Interval
    {
        let lo = tern!(self.lo < older.lo, i128::MIN, self.lo);
        let hi = tern!(self.hi > older.hi, i128::MAX, self.hi);
        Interval { lo, hi }
    }
}

/// What the check knows of a pointer, or of the pointers in a list.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub(crate) struct PtrInfo
{
    /// Whether it holds a block, null, or moved out.
    pub(crate) state: OwnerState,
    /// The places it can point into; none for an owner, whose block is
    /// its own.
    #[builder(default)]
    pub(crate) roots: Roots,
    /// The element offset from the start of the block.
    pub(crate) offset: Interval,
    /// The element count of the block, when known.
    pub(crate) len: Option<u64>,
    /// For a plain list of owners, the join of what its cells hold: the
    /// state of each owner added, and not known after a write through a
    /// cell, whose number the check does not follow.
    #[builder(default = OwnerState::NONE)]
    pub(crate) cells: OwnerState,
}

/// What the check knows of one slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SlotState
{
    /// An owner or a borrow.
    Ptr(PtrInfo),
    /// An integer.
    Int(Interval),
    /// A list that holds pointers: one fact for all its cells.
    List(PtrInfo),
    /// A slot the check does not follow.
    None,
}

/// What the check knows of each slot at one point of a routine.
#[derive(Debug, Clone, PartialEq, Eq, Default, Builder)]
pub(crate) struct State
{
    /// The global slots, in module order.
    pub(crate) globals: Vec<SlotState>,
    /// The local slots of the routine, in slot order. The check keeps a
    /// state for each node, so the slots share what a node leaves alone:
    /// a plain list of them would cost the slot count at each node.
    #[builder(default)]
    pub(crate) locals: Vector<SlotState>,
    /// The paths from a parameter or a global that a free, in the routine
    /// or in a call, left without a block on some path here, and no
    /// allocation filled again: the owner there can be null.
    #[builder(default)]
    pub(crate) unfilled: BTreeSet<Root>,
    /// Pairs of an integer local and a slice local where the integer is
    /// known to be below the count of the slice, after a test such as
    /// `န < s ၏ အကွက်အရေအတွက်`. A write to either slot ends the pair.
    #[builder(default)]
    pub(crate) below: BTreeSet<(Place, Place)>,
    /// The functions declared inside a function whose closures each slot
    /// can hold here; a slot with none is left out.
    #[builder(default)]
    pub(crate) closures: BTreeMap<Place, BTreeSet<u32>>,
}

/// The closures each slot can hold on either path: those of both.
///
/// # Arguments
///
/// * `a` - The closures of one path.
/// * `b` - The closures of the other.
///
/// # Returns
///
/// The joined closures.
fn join_closures(
    a: &BTreeMap<Place, BTreeSet<u32>>, b: &BTreeMap<Place, BTreeSet<u32>>,
) -> BTreeMap<Place, BTreeSet<u32>>
{
    let mut joined = a.clone();
    for (place, funcs) in b
    {
        joined.entry(*place).or_default().extend(funcs);
    }
    joined
}

impl State
{
    /// The fact of a slot.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The fact, [`SlotState::None`] for a slot out of range.
    pub(crate) fn get(&self, place: Place) -> &SlotState
    {
        let slot = match place
        {
            Place::Global(n) => self.globals.get(n as usize),
            Place::Local(n) => self.locals.get(n as usize),
        };
        slot.unwrap_or(&SlotState::None)
    }

    /// The pointer fact of a slot that holds pointers.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The fact of a pointer or a list of pointers, else `None`.
    pub(crate) fn ptr(&self, place: Place) -> Option<&PtrInfo>
    {
        match self.get(place)
        {
            SlotState::Ptr(info) | SlotState::List(info) => Some(info),
            SlotState::Int(_) | SlotState::None => None,
        }
    }

    /// The pointer fact of a slot that holds pointers, to change.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The fact of a pointer or a list of pointers, else `None`.
    pub(crate) fn ptr_mut(&mut self, place: Place) -> Option<&mut PtrInfo>
    {
        let slot = match place
        {
            Place::Global(n) => self.globals.get_mut(n as usize),
            Place::Local(n) => self.locals.get_mut(n as usize),
        };
        match slot?
        {
            SlotState::Ptr(info) | SlotState::List(info) => Some(info),
            SlotState::Int(_) | SlotState::None => None,
        }
    }

    /// Give a slot a new fact; a slot out of range is left alone.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    /// * `fact` - The new fact.
    pub(crate) fn set(&mut self, place: Place, fact: SlotState)
    {
        self.below.retain(|(n, s)| *n != place && *s != place);
        let slot = match place
        {
            Place::Global(n) => self.globals.get_mut(n as usize),
            Place::Local(n) => self.locals.get_mut(n as usize),
        };
        do_if!(let Some(slot) = slot, *slot = fact);
    }

    /// The facts at the start of a routine. The main program starts with
    /// each slot at null or zero. A function knows nothing of the
    /// globals but the roots top-level code gives the borrows; each
    /// parameter is not known, a borrow parameter pointing into what its
    /// argument points into; the other locals start blank.
    ///
    /// # Arguments
    ///
    /// * `module` - The module.
    /// * `routine` - The routine.
    /// * `global_roots` - The roots top-level code gives each global.
    ///
    /// # Returns
    ///
    /// The facts.
    pub(crate) fn at_start(
        module: &Module, routine: &Routine, global_roots: &[Roots],
    ) -> State
    {
        let global = |(n, slot): (usize, &Slot)| {
            ret_if!(routine.is_main, SlotState::blank(slot.kind));
            let roots = global_roots.get(n).cloned().unwrap_or_default();
            SlotState::unknown(slot.kind, roots)
        };
        let local = |(n, slot): (usize, &Slot)| {
            ret_if!(n as u32 >= routine.params, SlotState::blank(slot.kind));
            SlotState::unknown(slot.kind, param_roots(n as u32, slot.kind))
        };
        State::builder()
            .globals(module.globals.iter().enumerate().map(global).collect())
            .locals(routine.locals.iter().enumerate().map(local).collect())
            .build()
    }

    /// The facts of either state, per slot, and the paths either leaves
    /// without a block.
    ///
    /// # Arguments
    ///
    /// * `other` - The other state.
    ///
    /// # Returns
    ///
    /// The joined state.
    pub(crate) fn join(&self, other: &State) -> State
    {
        let both = |a: &[SlotState], b: &[SlotState]| {
            a.iter().zip(b).map(|(mine, theirs)| mine.join(theirs)).collect()
        };
        State::builder()
            .globals(both(&self.globals, &other.globals))
            .locals(merged(&self.locals, &other.locals, SlotState::join))
            .unfilled(self.unfilled.union(&other.unfilled).cloned().collect())
            .below(self.below.intersection(&other.below).copied().collect())
            .closures(join_closures(&self.closures, &other.closures))
            .build()
    }

    /// The state with each interval that changed since `older` widened to
    /// [`Interval::TOP`], so a loop reaches a fixpoint.
    ///
    /// # Arguments
    ///
    /// * `older` - The state at the last visit.
    ///
    /// # Returns
    ///
    /// The widened state.
    pub(crate) fn widen(&self, older: &State) -> State
    {
        let both = |a: &[SlotState], b: &[SlotState]| {
            a.iter().zip(b).map(|(mine, old)| mine.widen(old)).collect()
        };
        State::builder()
            .globals(both(&self.globals, &older.globals))
            .locals(merged(&self.locals, &older.locals, SlotState::widen))
            .unfilled(self.unfilled.clone())
            .below(self.below.clone())
            .closures(self.closures.clone())
            .build()
    }
}

/// The slots of two states put together one slot at a time, sharing each slot
/// that is the same in both.
///
/// # Arguments
///
/// * `a` - The slots of one state.
/// * `b` - The slots of the other.
/// * `pair` - How two facts of one slot go together.
///
/// # Returns
///
/// The slots.
fn merged(
    a: &Vector<SlotState>, b: &Vector<SlotState>,
    pair: impl Fn(&SlotState, &SlotState) -> SlotState,
) -> Vector<SlotState>
{
    let mut out = a.clone();
    let differ =
        a.iter().zip(b.iter()).enumerate().filter(|(_, (mine, theirs))| mine != theirs);
    let changes: Vec<(usize, SlotState)> =
        differ.map(|(n, (mine, theirs))| (n, pair(mine, theirs))).collect();
    do_for!((n, fact) in changes, out.set(n, fact));
    out
}

/// The roots of a borrow parameter: what the argument points into, which
/// lives past the call. A list parameter that holds borrows points through
/// its cells.
///
/// # Arguments
///
/// * `n` - The parameter number.
/// * `kind` - Its kind.
///
/// # Returns
///
/// The roots; none for a parameter that holds no borrow.
fn param_roots(n: u32, kind: DataKind) -> Roots
{
    let slot = Root::slot(Place::Local(n));
    ret_if!(!holds_address(kind), Roots::new());
    ret_if!(kind.is_aggregate(), Roots::from([slot.cell(None).block()]));
    Roots::from([slot.block()])
}

impl PtrInfo
{
    /// The facts of either pointer.
    ///
    /// # Arguments
    ///
    /// * `other` - The other fact.
    ///
    /// # Returns
    ///
    /// The union of the states and of the roots, the hull of the offsets,
    /// the count both agree on, and the union of the cell states.
    pub(crate) fn join(&self, other: &PtrInfo) -> PtrInfo
    {
        PtrInfo::builder()
            .state(self.state.join(other.state))
            .roots(self.roots.union(&other.roots).cloned().collect())
            .offset(self.offset.hull(other.offset))
            .maybe_len(self.len.filter(|_| self.len == other.len))
            .cells(self.cells.join(other.cells))
            .build()
    }

    /// The fact with its offset widened against an older fact.
    ///
    /// # Arguments
    ///
    /// * `older` - The fact at the last visit.
    ///
    /// # Returns
    ///
    /// The fact.
    fn widen(&self, older: &PtrInfo) -> PtrInfo
    {
        PtrInfo { offset: self.offset.widen(older.offset), ..self.clone() }
    }

    /// The pointer null.
    ///
    /// # Returns
    ///
    /// The fact.
    pub(crate) fn null() -> PtrInfo
    {
        PtrInfo::builder().state(OwnerState::NULL).offset(Interval::point(0)).build()
    }

    /// A pointer that can hold a block or null, at an offset not known.
    ///
    /// # Arguments
    ///
    /// * `roots` - The places it can point into.
    ///
    /// # Returns
    ///
    /// The fact.
    pub(crate) fn unknown(roots: Roots) -> PtrInfo
    {
        PtrInfo::builder()
            .state(OwnerState::UNKNOWN)
            .roots(roots)
            .offset(Interval::TOP)
            .cells(OwnerState::UNKNOWN)
            .build()
    }
}

impl SlotState
{
    /// The facts of either slot state.
    ///
    /// # Arguments
    ///
    /// * `other` - The other state of the slot.
    ///
    /// # Returns
    ///
    /// The joined state; [`SlotState::None`] when the two do not match.
    fn join(&self, other: &SlotState) -> SlotState
    {
        match (self, other)
        {
            (SlotState::Ptr(a), SlotState::Ptr(b)) => SlotState::Ptr(a.join(b)),
            (SlotState::List(a), SlotState::List(b)) => SlotState::List(a.join(b)),
            (SlotState::Int(a), SlotState::Int(b)) => SlotState::Int(a.hull(*b)),
            _ => SlotState::None,
        }
    }

    /// The state with each interval widened against an older state.
    ///
    /// # Arguments
    ///
    /// * `older` - The state at the last visit.
    ///
    /// # Returns
    ///
    /// The widened state.
    fn widen(&self, older: &SlotState) -> SlotState
    {
        match (self, older)
        {
            (SlotState::Ptr(a), SlotState::Ptr(b)) => SlotState::Ptr(a.widen(b)),
            (SlotState::List(a), SlotState::List(b)) => SlotState::List(a.widen(b)),
            (SlotState::Int(a), SlotState::Int(b)) => SlotState::Int(a.widen(*b)),
            _ => self.clone(),
        }
    }

    /// The fact a slot of `kind` holds before the program sets it: null
    /// or zero.
    ///
    /// # Arguments
    ///
    /// * `kind` - The slot kind.
    ///
    /// # Returns
    ///
    /// The fact.
    pub(crate) fn blank(kind: DataKind) -> SlotState
    {
        SlotState::of_kind(kind, PtrInfo::null(), Interval::point(0))
    }

    /// The fact of a slot of `kind` whose value is not known.
    ///
    /// # Arguments
    ///
    /// * `kind` - The slot kind.
    /// * `roots` - The places a borrow in it can point into.
    ///
    /// # Returns
    ///
    /// The fact.
    pub(crate) fn unknown(kind: DataKind, roots: Roots) -> SlotState
    {
        SlotState::of_kind(kind, PtrInfo::unknown(roots), Interval::TOP)
    }

    /// The fact of a slot of `kind`: `info` for a pointer or a list of
    /// pointers, `int` for an integer.
    ///
    /// # Arguments
    ///
    /// * `kind` - The slot kind.
    /// * `info` - The fact of a pointer.
    /// * `int` - The fact of an integer.
    ///
    /// # Returns
    ///
    /// The fact; [`SlotState::None`] for a kind the check does not follow.
    pub(crate) fn of_kind(kind: DataKind, info: PtrInfo, int: Interval) -> SlotState
    {
        ret_if!(kind.is_int(), SlotState::Int(int));
        // A function value is followed for whether it holds a function.
        ret_if!(kind.is_fn(), SlotState::Ptr(info));
        let words = kind.is_text() || kind.is_seq();
        let list = kind.is_aggregate() && !words;
        ret_if!(list && holds_ptr(kind), SlotState::List(info));
        tern!(holds_ptr(kind), SlotState::Ptr(info), SlotState::None)
    }
}

/// Whether the check follows the pointers of a kind: an owner, a borrow, a
/// slice, text or a plain list, or a counted list or a struct whose cells
/// or fields hold one at some depth. Raw addresses are not followed.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for a followed kind.
pub(crate) fn holds_ptr(kind: DataKind) -> bool
{
    ret_if!(let Some(list) = kind.as_list(), holds_ptr(list.cell));
    ret_if!(let Some(s) = kind.as_struct(), s.fields().iter().any(|(_, k)| holds_ptr(*k)));
    let words = kind.is_text() || kind.is_seq();
    words || (kind.is_ptr() && !kind.is_raw_ptr())
}

/// Whether a value of the kind holds an address the program wrote: a
/// pointer, or a counted list or a struct with one at some depth. Text and
/// plain lists own their cells, so those lend nothing.
///
/// # Arguments
///
/// * `kind` - The kind.
///
/// # Returns
///
/// `true` for such a kind.
pub(crate) fn holds_address(kind: DataKind) -> bool
{
    ret_if!(let Some(list) = kind.as_list(), holds_address(list.cell));
    ret_if!(let Some(s) = kind.as_struct(), s.fields().iter().any(|(_, k)| holds_address(*k)));
    kind.is_ptr() && !kind.is_raw_ptr()
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn join_is_bit_or_and_narrowing_picks_the_edge()
    {
        let held = OwnerState::HELD;
        assert_eq!(OwnerState::NULL.join(held), OwnerState::UNKNOWN);
        assert_eq!(held.join(held), held);
        assert!(OwnerState::UNKNOWN.can_hold() && OwnerState::UNKNOWN.possible_null());
        assert!(OwnerState::FRESH.can_hold() && OwnerState::FRESH.possible_null());
        assert!(!held.possible_null() && !OwnerState::NULL.can_hold());
        assert!(OwnerState::FRESH.join(held).shares_fresh());
        assert!(!OwnerState::UNKNOWN.shares_fresh());
        assert!(OwnerState::NULL.is_null() && !OwnerState::UNKNOWN.is_null());
        assert!(
            OwnerState::MOVED.is_moved() && !OwnerState::MOVED.join(held).is_moved()
        );
        assert_eq!(OwnerState::UNKNOWN.narrow(true), OwnerState::NULL);
        assert_eq!(OwnerState::UNKNOWN.narrow(false), held);
        assert_eq!(OwnerState::FRESH.narrow(false), held);
        assert_eq!(OwnerState::FRESH.narrow(true), OwnerState::NULL);
        assert_eq!(OwnerState::MOVED.narrow(true), OwnerState::MOVED);
        assert!(!OwnerState::NULL.narrow(false).can_hold());
    }

    #[test]
    fn interval_hull_and_arithmetic()
    {
        let a = Interval { lo: 0, hi: 4 };
        let b = Interval { lo: -2, hi: 3 };
        assert_eq!(a.hull(b), Interval { lo: -2, hi: 4 });
        assert_eq!(a.plus(b), Interval { lo: -2, hi: 7 });
        assert_eq!(a.minus(b), Interval { lo: -3, hi: 6 });
        assert_eq!(a.times(b), Interval { lo: -8, hi: 12 });
        assert_eq!(b.negate(), Interval { lo: -3, hi: 2 });
        assert_eq!(Interval::TOP.plus(Interval::point(1)), Interval::TOP);
        assert_eq!(Interval::point(i128::MAX).times(Interval::point(2)), Interval::TOP);
    }

    #[test]
    fn an_interval_past_the_range_of_its_kind_is_top()
    {
        use zakac_semcheck::kind::Width;
        let i8_kind = DataKind::Int(IntKind::new(true, Width::Bits(8)));
        let u8_kind = DataKind::Int(IntKind::new(false, Width::Bits(8)));
        let isize_kind = DataKind::Int(IntKind::new(true, Width::Platform));
        let range = |lo, hi| Interval { lo, hi };
        assert_eq!(range(-128, 127).fit(i8_kind, 64), range(-128, 127));
        assert_eq!(Interval::point(128).fit(i8_kind, 64), Interval::TOP);
        assert_eq!(Interval::point(-1).fit(u8_kind, 64), Interval::TOP);
        assert_eq!(Interval::point(255).fit(u8_kind, 64), Interval::point(255));
        assert_eq!(Interval::point(1 << 31).fit(isize_kind, 32), Interval::TOP);
        assert_eq!(
            Interval::point(1 << 31).fit(isize_kind, 64),
            Interval::point(1 << 31)
        );
        let u64_kind = DataKind::Int(IntKind::new(false, Width::Bits(64)));
        assert_eq!(
            Interval::of_int_kind(u64_kind.int_kind().unwrap(), 64).hi,
            i128::from(u64::MAX)
        );
    }

    #[test]
    fn widening_sends_each_moving_end_to_its_limit()
    {
        let int = |lo, hi| SlotState::Int(Interval { lo, hi });
        let older =
            State::builder().globals(vec![int(0, 0), int(3, 3), int(5, 5)]).build();
        let next =
            State::builder().globals(vec![int(0, 1), int(3, 3), int(4, 5)]).build();
        let joined = older.join(&next);
        assert_eq!(joined.globals, vec![int(0, 1), int(3, 3), int(4, 5)]);
        let wide = joined.widen(&older);
        let rising = int(0, i128::MAX);
        let falling = int(i128::MIN, 5);
        assert_eq!(wide.globals, vec![rising, int(3, 3), falling]);
    }
}
