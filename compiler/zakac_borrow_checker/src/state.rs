use std::collections::BTreeSet;

use bon::Builder;
use zakac_ir_builder::ir::Module;
use zakac_ir_builder::ir::Place;
use zakac_ir_builder::ir::Slot;
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
    /// `ဗလာ`.
    pub(crate) const NULL: OwnerState = OwnerState(1);
    /// A block.
    pub(crate) const HELD: OwnerState = OwnerState(2);
    /// A fresh allocation not compared with `ဗလာ`, so a block or `ဗလာ`.
    pub(crate) const FRESH: OwnerState = OwnerState(4);
    /// Nothing: the value moved out and left `ဗလာ`.
    pub(crate) const MOVED: OwnerState = OwnerState(8);
    /// A mark, not a fact: a call can have freed, resized or moved the
    /// block through an address of the slot, so the slot is not reported
    /// as a leak until the routine writes it again.
    pub(crate) const GIVEN: OwnerState = OwnerState(16);
    /// A block or `ဗလာ`, with nothing known.
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

    /// Whether the owner is `ဗလာ` on each path.
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

    /// Whether the owner can be `ဗလာ` though it was never compared with
    /// it, or is `ဗလာ` on some path.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::NULL`] or [`OwnerState::FRESH`].
    #[cfg(test)]
    pub(crate) const fn possible_null(self) -> bool
    {
        self.shares(OwnerState(OwnerState::NULL.0 | OwnerState::FRESH.0))
    }

    /// Whether the owner can hold an allocation not compared with `ဗလာ`
    /// since it was made.
    ///
    /// # Returns
    ///
    /// `true` with [`OwnerState::FRESH`].
    pub(crate) const fn shares_fresh(self) -> bool { self.shares(OwnerState::FRESH) }

    /// The state a read of the owner sees: moved out reads as `ဗလာ`, and
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

    /// The state on one edge of a test against `ဗလာ`.
    ///
    /// # Arguments
    ///
    /// * `null` - `true` on the edge where the owner is `ဗလာ`.
    ///
    /// # Returns
    ///
    /// On the `ဗလာ` edge: `ဗလာ` for each fact that allows it, moved out
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
    /// The range; an unsigned 128-bit kind stops at [`i128::MAX`].
    pub(crate) fn of_int_kind(kind: IntKind, platform: u16) -> Interval
    {
        let shift = 128 - u32::from(kind.bits(platform)).clamp(1, 128);
        ret_if!(
            kind.signed,
            Interval { lo: i128::MIN >> shift, hi: i128::MAX >> shift }
        );
        let hi = i128::try_from(u128::MAX >> shift).unwrap_or(i128::MAX);
        Interval { lo: 0, hi }
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

    /// This interval, or [`Interval::TOP`] when it differs from `older`.
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
        tern!(self == older, self, Interval::TOP)
    }
}

/// What the check knows of a pointer, or of the pointers in a list.
#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub(crate) struct PtrInfo
{
    /// Whether it holds a block, `ဗလာ`, or moved out.
    pub(crate) state: OwnerState,
    /// The places it can point into; none for an owner, whose block is
    /// its own.
    #[builder(default)]
    pub(crate) roots: Roots,
    /// The element offset from the start of the block.
    pub(crate) offset: Interval,
    /// The element count of the block, when known.
    pub(crate) len: Option<u64>,
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
    /// The local slots of the routine, in slot order.
    #[builder(default)]
    pub(crate) locals: Vec<SlotState>,
    /// The paths from a parameter or a global that a free, in the routine
    /// or in a call, left without a block on some path here, and no
    /// allocation filled again: the owner there can be `ဗလာ`.
    #[builder(default)]
    pub(crate) unfilled: BTreeSet<Root>,
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
        let slot = match place
        {
            Place::Global(n) => self.globals.get_mut(n as usize),
            Place::Local(n) => self.locals.get_mut(n as usize),
        };
        do_if!(let Some(slot) = slot, *slot = fact);
    }

    /// The facts at the start of a routine. The main program starts with
    /// each slot at `ဗလာ` or zero. A function knows nothing of the
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
            .locals(both(&self.locals, &other.locals))
            .unfilled(self.unfilled.union(&other.unfilled).cloned().collect())
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
            .locals(both(&self.locals, &older.locals))
            .unfilled(self.unfilled.clone())
            .build()
    }
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
    ret_if!(!kind.holds_borrow(), Roots::new());
    ret_if!(
        kind.is_aggregate() && !kind.is_slice(),
        Roots::from([slot.cell(None).block()])
    );
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
    /// and the count both agree on.
    pub(crate) fn join(&self, other: &PtrInfo) -> PtrInfo
    {
        PtrInfo::builder()
            .state(self.state.join(other.state))
            .roots(self.roots.union(&other.roots).cloned().collect())
            .offset(self.offset.hull(other.offset))
            .maybe_len(self.len.filter(|_| self.len == other.len))
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

    /// The pointer `ဗလာ`.
    ///
    /// # Returns
    ///
    /// The fact.
    pub(crate) fn null() -> PtrInfo
    {
        PtrInfo::builder().state(OwnerState::NULL).offset(Interval::point(0)).build()
    }

    /// A pointer that can hold a block or `ဗလာ`, at an offset not known.
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

    /// The fact a slot of `kind` holds before the program sets it: `ဗလာ`
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
        let list = kind.is_aggregate() && !kind.is_slice();
        ret_if!(list && holds_ptr(kind), SlotState::List(info));
        tern!(holds_ptr(kind), SlotState::Ptr(info), SlotState::None)
    }
}

/// Whether the check follows the pointers of a kind: an owner, a borrow or
/// a slice, or a list or a struct whose cells or fields hold one at some
/// depth. Raw addresses are not followed.
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
    kind.is_slice() || (kind.is_ptr() && !kind.is_raw_ptr())
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
        let u128_kind = DataKind::Int(IntKind::new(false, Width::Bits(128)));
        assert_eq!(
            Interval::of_int_kind(u128_kind.int_kind().unwrap(), 64).hi,
            i128::MAX
        );
    }

    #[test]
    fn widening_sends_a_moving_interval_to_top()
    {
        let int = |lo, hi| SlotState::Int(Interval { lo, hi });
        let older = State::builder().globals(vec![int(0, 0), int(3, 3)]).build();
        let next = State::builder().globals(vec![int(0, 1), int(3, 3)]).build();
        let joined = older.join(&next);
        assert_eq!(joined.globals, vec![int(0, 1), int(3, 3)]);
        let wide = joined.widen(&older);
        assert_eq!(wide.globals, vec![SlotState::Int(Interval::TOP), int(3, 3)]);
    }
}
