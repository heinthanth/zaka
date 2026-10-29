use std::collections::BTreeSet;
use std::fmt;

use bon::Builder;
use serde::Deserialize;
use serde::Serialize;
use zakac_ir_builder::ir::Func;
use zakac_ir_builder::ir::Place;
use zakac_rust_sugars::do_if;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;

/// The most steps a path holds. A longer path is cut to a prefix, which
/// reaches more, so the cut is sound.
pub const MAX_PATH: usize = 4;

/// One step from a slot toward what it reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum Seg
{
    /// The block an owner pointer holds.
    Block,
    /// One cell of a list; `None` is a cell that is not known, so the whole
    /// list.
    Cell(Option<u32>),
}

impl fmt::Display for Seg
{
    /// Write the step as `block`, `cell[3]` or `cell[*]`.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The result of the write.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Seg::Block => write!(f, "block"),
            Seg::Cell(Some(n)) => write!(f, "cell[{n}]"),
            Seg::Cell(None) => write!(f, "cell[*]"),
        }
    }
}

/// What a pointer reaches: a slot and the steps from it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Serialize, Deserialize, Builder)]
pub struct Root
{
    /// The slot the path starts at.
    pub place: Place,
    /// The steps from the slot, at most [`MAX_PATH`].
    pub path: Vec<Seg>,
}

impl Root
{
    /// The path to a slot itself.
    ///
    /// # Arguments
    ///
    /// * `place` - The slot.
    ///
    /// # Returns
    ///
    /// The path with no step.
    pub fn slot(place: Place) -> Self
    {
        Self::builder().place(place).path(Vec::new()).build()
    }

    /// The path one step further. A path at [`MAX_PATH`] stalls: it stands
    /// for each deeper path too.
    ///
    /// # Arguments
    ///
    /// * `seg` - The step.
    ///
    /// # Returns
    ///
    /// The longer path.
    fn step(&self, seg: Seg) -> Self
    {
        let mut next = self.clone();
        do_if!(next.path.len() < MAX_PATH, next.path.push(seg));
        next
    }

    /// The path to the block the pointer at this path holds.
    ///
    /// # Returns
    ///
    /// The path with a [`Seg::Block`] step.
    pub fn block(&self) -> Self { self.step(Seg::Block) }

    /// The path to one cell of the list at this path.
    ///
    /// # Arguments
    ///
    /// * `index` - The cell number from 0, or `None` when it is not known.
    ///
    /// # Returns
    ///
    /// The path with a [`Seg::Cell`] step.
    pub fn cell(&self, index: Option<u32>) -> Self { self.step(Seg::Cell(index)) }

    /// The path with more steps after it.
    ///
    /// # Arguments
    ///
    /// * `steps` - The steps, in order.
    ///
    /// # Returns
    ///
    /// The longer path, stalled at [`MAX_PATH`].
    pub fn then(&self, steps: &[Seg]) -> Self
    {
        steps.iter().fold(self.clone(), |root, &seg| root.step(seg))
    }

    /// Whether a free at this path ends a loan on `loan`: both start at one
    /// slot and this path leads to `loan` or to a place `loan` is inside.
    ///
    /// # Arguments
    ///
    /// * `loan` - The path a borrow holds.
    ///
    /// # Returns
    ///
    /// `true` when this path is a prefix of `loan`; a cell that is not
    /// known matches each cell.
    pub fn reaches(&self, loan: &Root) -> bool
    {
        ret_false!(self.place != loan.place || self.path.len() > loan.path.len());
        self.path.iter().zip(&loan.path).all(|(a, b)| same_step(*a, *b))
    }

    /// Whether the two paths share a place: one reaches the other.
    ///
    /// # Arguments
    ///
    /// * `other` - The other path.
    ///
    /// # Returns
    ///
    /// `true` when either path is a prefix of the other.
    pub fn overlaps(&self, other: &Root) -> bool
    {
        self.reaches(other) || other.reaches(self)
    }

    /// Whether the path starts at a parameter.
    ///
    /// # Arguments
    ///
    /// * `params` - The number of parameters of the function.
    ///
    /// # Returns
    ///
    /// `true` for a local below the parameter count.
    pub fn is_param(&self, params: u32) -> bool
    {
        matches!(self.place, Place::Local(n) if n < params)
    }

    /// Whether the path starts at a global.
    ///
    /// # Returns
    ///
    /// `true` for a global slot.
    pub fn is_global(&self) -> bool { matches!(self.place, Place::Global(_)) }

    /// Whether a caller can name the path: it starts at a parameter or at
    /// a global.
    ///
    /// # Arguments
    ///
    /// * `params` - The number of parameters of the function.
    ///
    /// # Returns
    ///
    /// `true` for a path from a parameter or a global.
    pub fn is_outer(&self, params: u32) -> bool
    {
        self.is_param(params) || self.is_global()
    }
}

/// Whether two steps can lead to the same place.
///
/// # Arguments
///
/// * `a` - One step.
/// * `b` - The other step.
///
/// # Returns
///
/// `true` for equal steps, and for two cells when one number is not known.
fn same_step(a: Seg, b: Seg) -> bool
{
    match (a, b)
    {
        (Seg::Cell(None), Seg::Cell(_)) | (Seg::Cell(_), Seg::Cell(None)) => true,
        _ => a == b,
    }
}

impl fmt::Display for Root
{
    /// Write the slot, then each step after a dot.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The result of the write.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.place)?;
        self.path.iter().try_for_each(|seg| write!(f, ".{seg}"))
    }
}

/// What the check knows of a function, for the checks at its calls.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize, Builder)]
pub struct FnDigest
{
    /// The function name.
    #[builder(into)]
    pub name: String,
    /// The number of parameters; `l0` to the one before this are the
    /// parameters.
    pub params: u32,
    /// What the function frees, resizes or moves out, as paths from its
    /// parameters and from the globals.
    pub frees: Vec<Root>,
    /// The parameters the result can point into.
    #[builder(default)]
    pub result_from: Vec<u32>,
    /// What the result can point into, as paths from the parameters it
    /// borrows through and from the globals.
    #[builder(default)]
    pub result_roots: Vec<Root>,
    /// The result paths the function itself frees, resizes or moves
    /// through before it returns: a borrow on such a path that lives to
    /// the return was made after it, so it points into the new block. A
    /// call takes out those an `unfilled` path reaches, since the block
    /// there need not be filled again.
    #[builder(default)]
    pub renewed: Vec<Root>,
    /// The paths from the parameters and the globals the function frees
    /// and does not fill again on some path to its exit: the owner there
    /// can be `ဗလာ` after the call.
    #[builder(default)]
    pub unfilled: Vec<Root>,
    /// The paths from the parameters and the globals the function reads
    /// or writes through, itself or in a call, in whatever order against
    /// its frees: a global one that reaches a block the function frees
    /// through a parameter is a read of freed space inside the call.
    #[builder(default)]
    pub reads: Vec<Root>,
    /// Whether the result points into nothing that dies with the call.
    #[builder(default = true)]
    pub result_roots_ok: bool,
}

impl FnDigest
{
    /// The digest that knows nothing: no frees, a result from no
    /// parameter. The first round of the fixpoint over a call loop starts
    /// from it.
    ///
    /// # Arguments
    ///
    /// * `func` - The function.
    ///
    /// # Returns
    ///
    /// The digest.
    pub fn blank(func: &Func) -> Self
    {
        Self::builder().name(&func.name).params(func.params).frees(Vec::new()).build()
    }

    /// What either digest knows: the union of what each frees and of
    /// what each result can point into. A round of the fixpoint over a
    /// call loop keeps what the rounds before it found, so the rounds
    /// grow and never shrink, and the loop ends.
    ///
    /// # Arguments
    ///
    /// * `other` - The other digest of the same function.
    ///
    /// # Returns
    ///
    /// The joined digest.
    pub fn join(&self, other: &FnDigest) -> FnDigest
    {
        let ok = self.result_roots_ok && other.result_roots_ok;
        FnDigest::builder()
            .name(&self.name)
            .params(self.params)
            .frees(union(&self.frees, &other.frees))
            .result_from(union(&self.result_from, &other.result_from))
            .result_roots(union(&self.result_roots, &other.result_roots))
            .renewed(union(&self.renewed, &other.renewed))
            .unfilled(union(&self.unfilled, &other.unfilled))
            .reads(union(&self.reads, &other.reads))
            .result_roots_ok(ok)
            .build()
    }

    /// Whether the function frees, resizes or moves out something reached
    /// through a parameter.
    ///
    /// # Arguments
    ///
    /// * `n` - The parameter number from 0.
    ///
    /// # Returns
    ///
    /// `true` when a freed path starts at the parameter.
    pub fn frees_param(&self, n: u32) -> bool
    {
        self.frees.iter().any(|root| root.place == Place::Local(n))
    }

    /// Whether the function frees, resizes or moves out something reached
    /// through a global.
    ///
    /// # Arguments
    ///
    /// * `g` - The global number.
    ///
    /// # Returns
    ///
    /// `true` when a freed path starts at the global.
    pub fn frees_global(&self, g: u32) -> bool
    {
        self.frees.iter().any(|root| root.place == Place::Global(g))
    }
}

/// The items of two lists, each once, in order.
///
/// # Arguments
///
/// * `a` - One list.
/// * `b` - The other list.
///
/// # Returns
///
/// The sorted union.
fn union<T: Ord + Clone>(a: &[T], b: &[T]) -> Vec<T>
{
    let all: BTreeSet<&T> = a.iter().chain(b).collect();
    all.into_iter().cloned().collect()
}

/// Write the roots with commas, or `none` for no root.
///
/// # Arguments
///
/// * `f` - The formatter.
/// * `roots` - The roots.
///
/// # Returns
///
/// The result of the write.
fn write_roots(f: &mut fmt::Formatter<'_>, roots: &[&Root]) -> fmt::Result
{
    ret_if!(roots.is_empty(), write!(f, "none"));
    let words: Vec<String> = roots.iter().map(|root| root.to_string()).collect();
    write!(f, "{}", words.join(", "))
}

impl fmt::Display for FnDigest
{
    /// Write one line: the name, the parameters it frees through, then the
    /// globals it frees.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// The result of the write.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        let roots = self.frees.iter();
        let params: Vec<&Root> =
            roots.clone().filter(|root| root.is_param(self.params)).collect();
        let globals: Vec<&Root> = roots.filter(|root| root.is_global()).collect();
        write!(f, "{}: params ", self.name)?;
        write_roots(f, &params)?;
        write!(f, "; globals ")?;
        write_roots(f, &globals)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn block_path_reaches_deeper_paths()
    {
        let slot = Root::slot(Place::Local(0));
        let block = slot.block();
        let inner = block.block();
        assert!(slot.reaches(&block));
        assert!(block.reaches(&inner));
        assert!(!inner.reaches(&block));
        assert!(!block.reaches(&slot));
        assert!(!block.reaches(&Root::slot(Place::Local(1)).block()));
        assert!(block.overlaps(&inner) && inner.overlaps(&block));
    }

    #[test]
    fn cell_not_known_matches_each_cell()
    {
        let list = Root::slot(Place::Global(2));
        let first = list.cell(Some(0)).block();
        let second = list.cell(Some(1)).block();
        let some = list.cell(None).block();
        assert!(!first.reaches(&second));
        assert!(some.reaches(&first) && first.reaches(&some));
        assert!(!first.reaches(&list.cell(Some(0))));
    }

    #[test]
    fn long_path_stalls_at_the_cap()
    {
        let mut root = Root::slot(Place::Local(0));
        (0..MAX_PATH + 2).for_each(|_| root = root.block());
        assert_eq!(root.path.len(), MAX_PATH);
        assert!(root.reaches(&root.block()));
    }

    #[test]
    fn digest_frees_name_the_start_slot()
    {
        let frees =
            vec![Root::slot(Place::Local(1)).block(), Root::slot(Place::Global(0))];
        let digest = FnDigest::builder().name("f").params(2).frees(frees).build();
        assert!(digest.frees_param(1) && !digest.frees_param(0));
        assert!(digest.frees_global(0) && !digest.frees_global(1));
    }
}
