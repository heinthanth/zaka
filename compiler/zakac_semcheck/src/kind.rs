use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use serde::Deserialize;
use serde::Serialize;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_rust_sugars::ret_false;
use zakac_rust_sugars::ret_if;
use zakac_rust_sugars::ret_true;
use zakac_rust_sugars::tern;

use crate::structs::StructKind;
use crate::structs::StructTable;
use crate::structs::parse_sig;

/// Bit counts an integer kind can have.
const INT_BITS: &[u16] = &[8, 16, 32, 64, 128];
/// Bit counts a floating-point kind can have.
const FLOAT_BITS: &[u16] = &[16, 32, 64, 128];

/// Width of an integer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Width
{
    /// Platform size; the check options give its bit count.
    Platform,
    /// A fixed bit count.
    Bits(u16),
}

/// Sign and width of an integer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntKind
{
    /// `true` for a signed integer.
    pub signed: bool,
    /// The width.
    pub width: Width,
}

impl IntKind
{
    /// Make an integer kind.
    ///
    /// # Arguments
    ///
    /// * `signed` - `true` for a signed integer.
    /// * `width` - The width.
    ///
    /// # Returns
    ///
    /// The kind.
    pub const fn new(signed: bool, width: Width) -> Self { Self { signed, width } }

    /// The bit count of the kind.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count.
    pub fn bits(self, platform: u16) -> u16
    {
        match self.width
        {
            Width::Platform => platform,
            Width::Bits(n) => n,
        }
    }
}

/// The kind at the bottom of a pointer kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Base
{
    /// An integer.
    Int(IntKind),
    /// A floating-point number with this bit count.
    Float(u16),
    /// A character.
    Char,
    /// A boolean.
    Bool,
    /// A bare `လိပ်စာ`, whose target kind is not known.
    Opaque,
    /// A list, for a pointer to a whole list.
    List(&'static ListKind),
    /// A struct, for a pointer to a whole struct.
    Struct(&'static StructKind),
    /// A slice, for a pointer to a slice value.
    Slice(&'static SliceKind),
}

impl Base
{
    /// The plain kind of this base.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for [`Base::Opaque`], which is an address itself.
    pub fn scalar(self) -> Option<DataKind>
    {
        match self
        {
            Base::Int(n) => Some(DataKind::Int(n)),
            Base::Float(bits) => Some(DataKind::Float(bits)),
            Base::Char => Some(DataKind::Char),
            Base::Bool => Some(DataKind::Bool),
            Base::Opaque => None,
            Base::List(l) => Some(DataKind::List(l)),
            Base::Struct(s) => Some(DataKind::Struct(s)),
            Base::Slice(s) => Some(DataKind::Slice(s)),
        }
    }
}

/// A list kind: `len` cells of one kind, next to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListKind
{
    /// The kind of each cell.
    pub cell: DataKind,
    /// The number of cells; 0 while the initializer has not given it.
    pub len: u32,
}

/// Each list kind made so far, so a kind is made once and remains `Copy`.
static LISTS: OnceLock<Mutex<HashSet<&'static ListKind>>> = OnceLock::new();

/// A slice kind: a borrowed run of cells of one kind, whose count is known at
/// run time. A slice value is two words: the address of the first cell, then
/// the count as a `usize`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SliceKind
{
    /// The kind of each cell.
    pub cell: DataKind,
}

/// Each slice kind made so far, so a kind is made once and remains `Copy`.
static SLICES: OnceLock<Mutex<HashSet<&'static SliceKind>>> = OnceLock::new();

/// A pointer kind: `depth` levels of address over a base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtrKind
{
    /// The number of address levels, at least 1. An opaque base counts
    /// itself, so `ptr` has depth 1 and `*ptr` depth 2.
    pub depth: u8,
    /// The kind at the bottom.
    pub base: Base,
    /// Whether the outer level is a borrow: it reads a block that an owner
    /// holds.
    pub borrowed: bool,
    /// The raw levels as a bitmask: bit 0 is the level nearest the base and
    /// bit `depth - 1` the outer one. A level without its bit is an owner.
    pub raw: u8,
}

impl PtrKind
{
    /// The owner kind `depth` levels over `base`.
    ///
    /// # Arguments
    ///
    /// * `depth` - The number of address levels, at least 1.
    /// * `base` - The kind at the bottom.
    ///
    /// # Returns
    ///
    /// The kind with no borrow mark and no raw level.
    pub const fn owner(depth: u8, base: Base) -> PtrKind
    {
        PtrKind { depth, base, borrowed: false, raw: 0 }
    }

    /// The mask of the outer level in [`PtrKind::raw`].
    ///
    /// # Returns
    ///
    /// The bit, or 0 when the depth is past the mask width.
    fn outer_bit(self) -> u8
    {
        let shift = u32::from(self.depth.saturating_sub(1));
        1u8.checked_shl(shift).unwrap_or(0)
    }

    /// Whether the outer level is a borrow.
    ///
    /// # Returns
    ///
    /// `true` for a borrow.
    pub fn is_borrowed(self) -> bool { self.borrowed }

    /// Whether the outer level is a raw address.
    ///
    /// # Returns
    ///
    /// `true` for a raw outer level.
    pub fn is_raw_outer(self) -> bool { self.raw & self.outer_bit() != 0 }

    /// Whether the outer level is an owner.
    ///
    /// # Returns
    ///
    /// `true` when the level is neither a borrow nor raw.
    pub fn is_owner(self) -> bool { !self.borrowed && !self.is_raw_outer() }

    /// The kind with its outer level turned into a borrow.
    ///
    /// # Returns
    ///
    /// The borrowed kind; a raw outer level cannot be borrowed, so it is
    /// returned as it is.
    pub fn as_borrowed(self) -> PtrKind
    {
        PtrKind { borrowed: !self.is_raw_outer(), ..self }
    }

    /// The kind with its outer level turned into an owner.
    ///
    /// # Returns
    ///
    /// The kind without a borrow mark.
    pub fn as_owner(self) -> PtrKind { PtrKind { borrowed: false, ..self } }

    /// The kind with its outer level turned into a raw address.
    ///
    /// # Returns
    ///
    /// The kind with the outer raw bit set and no borrow mark.
    pub fn with_outer_raw(self) -> PtrKind
    {
        PtrKind { borrowed: false, raw: self.raw | self.outer_bit(), ..self }
    }

    /// The kind without the borrow mark, which the shape of the block
    /// does not depend on.
    ///
    /// # Returns
    ///
    /// The owner flavour of the outer level, each raw level kept.
    pub fn shape(self) -> PtrKind { self.as_owner() }
}

/// A data kind as the checker sees it: a written kind with its defaults
/// filled in. Artifacts store it as its short name, such as `i8`, `usize`,
/// `*i32` or `ptr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum DataKind
{
    /// An integer.
    Int(IntKind),
    /// A floating-point number with this bit count.
    Float(u16),
    /// A character.
    Char,
    /// A boolean.
    Bool,
    /// A pointer.
    Ptr(PtrKind),
    /// The kind of `ဗလာ`, which converts to each pointer kind.
    Null,
    /// A list of cells.
    List(&'static ListKind),
    /// A struct of named fields.
    Struct(&'static StructKind),
    /// A slice: the address of a run of cells and their count.
    Slice(&'static SliceKind),
}

impl DataKind
{
    /// `ကိန်းပြည့်`, the default integer kind.
    pub const ISIZE: DataKind = DataKind::Int(IntKind::new(true, Width::Platform));
    /// `လက္ခဏာမဲ့ ကိန်းပြည့်`
    pub const USIZE: DataKind = DataKind::Int(IntKind::new(false, Width::Platform));
    /// `ဒဿမကိန်း`, the default floating-point kind.
    pub const F64: DataKind = DataKind::Float(64);
    /// `လိပ်စာ`, the opaque pointer.
    pub const OPAQUE: DataKind = DataKind::Ptr(PtrKind::owner(1, Base::Opaque));

    /// The data kind a written kind phrase names.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind phrase.
    /// * `structs` - The kind of each declared struct.
    ///
    /// # Returns
    ///
    /// The kind; a missing sign means signed, a missing bit count means
    /// platform size for integers and 64 for floats. Each pointer level of
    /// the phrase adds one address level. A struct name that is not declared
    /// stands for `isize`; the check reports it.
    pub fn from_ast(kind: &Kind, structs: &StructTable) -> DataKind
    {
        let width = kind.bits.map_or(Width::Platform, Width::Bits);
        let base = match kind.base
        {
            BaseKind::Int =>
            {
                Base::Int(IntKind::new(kind.signed.unwrap_or(true), width))
            }
            BaseKind::Float => Base::Float(kind.bits.unwrap_or(64)),
            BaseKind::Char => Base::Char,
            BaseKind::Bool => Base::Bool,
            BaseKind::Opaque => Base::Opaque,
            BaseKind::Named => named_base(kind, structs),
        };
        let skip = usize::from(kind.base == BaseKind::Opaque);
        let plain = DataKind::over(base, u8::try_from(skip).unwrap_or(1));
        let first_raw = skip == 1 && kind.wraps.first() == Some(&Wrap::RawPtr);
        let plain = tern!(first_raw, plain.with_outer_raw(), plain);
        let marked = tern!(
            kind.borrowed,
            kind.wraps.iter().rposition(|w| matches!(w, Wrap::Ptr | Wrap::RawPtr)),
            None
        );
        let plain = tern!(marked.is_some_and(|i| i < skip), plain.as_borrowed(), plain);
        let wraps = kind.wraps.iter().enumerate().skip(skip);
        wraps.fold(plain, |inner, (at, w)| match w
        {
            Wrap::Ptr => inner.pointer_to().marked_if(marked == Some(at)),
            Wrap::RawPtr => inner.raw_pointer_to(),
            Wrap::List(Cells::Fixed(n)) => DataKind::list_of(inner, *n),
            Wrap::List(Cells::Inferred) => DataKind::list_of(inner, 0),
            Wrap::Slice => DataKind::slice_of(inner),
        })
    }

    /// The kind as a borrow when `mark` holds.
    ///
    /// # Arguments
    ///
    /// * `mark` - Whether the borrow mark applies.
    ///
    /// # Returns
    ///
    /// [`DataKind::as_borrowed`] of the kind, or the kind itself.
    fn marked_if(self, mark: bool) -> DataKind { tern!(mark, self.as_borrowed(), self) }

    /// The kind with its outer address level turned into a raw one.
    ///
    /// # Returns
    ///
    /// The kind for a pointer, the kind itself for each other kind.
    pub(crate) fn with_outer_raw(self) -> DataKind
    {
        tern!(let DataKind::Ptr(p) = self, DataKind::Ptr(p.with_outer_raw()), self)
    }

    /// The list kind of `len` cells of `cell`. Each list kind is made once
    /// and kept for the rest of the run.
    ///
    /// # Arguments
    ///
    /// * `cell` - The kind of each cell.
    /// * `len` - The number of cells; 0 while the initializer has not given it.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn list_of(cell: DataKind, len: u32) -> DataKind
    {
        let wanted = ListKind { cell, len };
        let lists = LISTS.get_or_init(Mutex::default);
        let mut lists = lists.lock().unwrap_or_else(PoisonError::into_inner);
        ret_if!(let Some(found) = lists.get(&wanted), DataKind::List(found));
        let made: &'static ListKind = Box::leak(Box::new(wanted));
        lists.insert(made);
        DataKind::List(made)
    }

    /// The slice kind of cells of `cell`. Each slice kind is made once and
    /// kept for the rest of the run.
    ///
    /// # Arguments
    ///
    /// * `cell` - The kind of each cell.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn slice_of(cell: DataKind) -> DataKind
    {
        let wanted = SliceKind { cell };
        let slices = SLICES.get_or_init(Mutex::default);
        let mut slices = slices.lock().unwrap_or_else(PoisonError::into_inner);
        ret_if!(let Some(found) = slices.get(&wanted), DataKind::Slice(found));
        let made: &'static SliceKind = Box::leak(Box::new(wanted));
        slices.insert(made);
        DataKind::Slice(made)
    }

    /// The slice part of the kind.
    ///
    /// # Returns
    ///
    /// The cell kind, or `None` for a kind that is no slice.
    pub fn as_slice(self) -> Option<&'static SliceKind>
    {
        tern!(let DataKind::Slice(s) = self, Some(s), None)
    }

    /// Whether the kind is a slice.
    ///
    /// # Returns
    ///
    /// `true` for a slice kind.
    pub fn is_slice(self) -> bool { matches!(self, DataKind::Slice(_)) }

    /// The kind of each cell a value of the kind holds or views: the cells of
    /// a list or of a slice.
    ///
    /// # Returns
    ///
    /// The cell kind, or `None` for a kind that is no list and no slice.
    pub fn cell_kind(self) -> Option<DataKind>
    {
        self.as_list().map(|l| l.cell).or_else(|| self.as_slice().map(|s| s.cell))
    }

    /// The list part of the kind.
    ///
    /// # Returns
    ///
    /// The cell kind and count, or `None` for a kind that is no list.
    pub fn as_list(self) -> Option<&'static ListKind>
    {
        tern!(let DataKind::List(l) = self, Some(l), None)
    }

    /// Whether the kind is a list.
    ///
    /// # Returns
    ///
    /// `true` for a list kind.
    pub fn is_list(self) -> bool { matches!(self, DataKind::List(_)) }

    /// The struct part of the kind.
    ///
    /// # Returns
    ///
    /// The struct, or `None` for a kind that is no struct.
    pub fn as_struct(self) -> Option<&'static StructKind>
    {
        tern!(let DataKind::Struct(s) = self, Some(s), None)
    }

    /// Whether the kind is a struct.
    ///
    /// # Returns
    ///
    /// `true` for a struct kind.
    pub fn is_struct(self) -> bool { matches!(self, DataKind::Struct(_)) }

    /// Whether a value of the kind is a block of other values: a list, a
    /// struct, or the two words of a slice. Such a value lives in mem, and a
    /// duplicate takes the whole block.
    ///
    /// # Returns
    ///
    /// `true` for a list, a struct or a slice kind.
    pub fn is_aggregate(self) -> bool
    {
        self.is_list() || self.is_struct() || self.is_slice()
    }

    /// Whether some list level of the kind has no cell count so far.
    ///
    /// # Returns
    ///
    /// `true` when a count is still 0.
    pub fn has_open_count(self) -> bool
    {
        ret_if!(let Some(s) = self.as_slice(), s.cell.has_open_count());
        self.as_list().is_some_and(|l| l.len == 0 || l.cell.has_open_count())
    }

    /// Fill the open counts of this list kind from `found`, a kind with the
    /// same shape.
    ///
    /// # Arguments
    ///
    /// * `found` - The kind that gives the counts.
    ///
    /// # Returns
    ///
    /// The kind with its counts filled, or `None` when the shapes differ.
    pub fn fill_counts(self, found: DataKind) -> Option<DataKind>
    {
        if let (Some(want), Some(have)) = (self.as_slice(), found.as_slice())
        {
            return Some(DataKind::slice_of(want.cell.fill_counts(have.cell)?));
        }
        let (Some(want), Some(have)) = (self.as_list(), found.as_list())
        else
        {
            return tern!(self.has_open_count(), None, Some(self));
        };
        ret_if!(want.len != 0 && want.len != have.len, None);
        Some(DataKind::list_of(want.cell.fill_counts(have.cell)?, have.len))
    }

    /// The kind `depth` address levels over `base`.
    ///
    /// # Arguments
    ///
    /// * `base` - The kind at the bottom.
    /// * `depth` - The number of address levels.
    ///
    /// # Returns
    ///
    /// The plain kind of `base` for depth 0, else a pointer kind; an opaque
    /// base has at least one level.
    pub fn over(base: Base, depth: u8) -> DataKind
    {
        let plain = base.scalar().filter(|_| depth == 0);
        plain.unwrap_or(DataKind::Ptr(PtrKind::owner(depth.max(1), base)))
    }

    /// Read a short name such as `i8`, `usize`, `f32`, `char`, `*i32`, `ptr`,
    /// `&*i32`, `~i32`, `rawptr` or `null`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for an unknown name, for a borrow mark on a kind
    /// with no address level and for a borrowed raw level.
    pub fn parse(name: &str) -> Option<DataKind>
    {
        ret_if!(name == "null", Some(DataKind::Null));
        ret_if!(name.starts_with("&["), parse_base(name)?.0.scalar());
        let (borrowed, name) = tern!(
            let Some(rest) = name.strip_prefix('&'),
            (true, rest),
            (false, name)
        );
        let rest = name.trim_start_matches(['*', '~']);
        let marks = &name[..name.len() - rest.len()];
        let (base, base_raw) = parse_base(rest)?;
        let opaque = base == Base::Opaque;
        let depth = u8::try_from(marks.len() + usize::from(opaque)).ok()?;
        ret_if!(depth == 0, tern!(borrowed, None, base.scalar()));
        let mut ptr = PtrKind::owner(depth, base);
        ptr.raw = u8::from(base_raw);
        for (at, mark) in marks.chars().enumerate()
        {
            let level =
                u8::try_from(marks.len() - 1 - at + usize::from(opaque)).ok()?;
            let bit = 1u8.checked_shl(u32::from(level));
            ptr.raw |= tern!(mark == '~', bit?, 0);
        }
        ret_if!(borrowed && ptr.is_raw_outer(), None);
        Some(DataKind::Ptr(PtrKind { borrowed, ..ptr }))
    }

    /// The short name, for messages.
    ///
    /// # Returns
    ///
    /// The name, such as `i8`, `usize`, `f64`, `char`, `bool`, `*i32`,
    /// `ptr` or `null`; a struct is its name alone.
    pub fn name(self) -> String { self.spell(false) }

    /// The name that reads back with [`DataKind::parse`], for artifacts.
    ///
    /// # Returns
    ///
    /// The short name, with each struct written as its signature.
    pub fn sig_name(self) -> String { self.spell(true) }

    /// The name of the kind.
    ///
    /// # Arguments
    ///
    /// * `full` - Whether a struct is written as its signature.
    ///
    /// # Returns
    ///
    /// The name.
    fn spell(self, full: bool) -> String
    {
        match self
        {
            DataKind::Int(n) => int_name(n),
            DataKind::Float(bits) => format!("f{bits}"),
            DataKind::Char => "char".to_string(),
            DataKind::Bool => "bool".to_string(),
            DataKind::Ptr(p) => ptr_name(p, full),
            DataKind::Null => "null".to_string(),
            DataKind::List(l) => list_name(l, full),
            DataKind::Struct(s) => tern!(full, s.sig(), &s.name).to_string(),
            DataKind::Slice(s) => format!("&[]{}", s.cell.spell(full)),
        }
    }

    /// The address levels and the base of the kind.
    ///
    /// # Returns
    ///
    /// Depth 0 and the kind itself for a plain kind, the parts of a pointer
    /// kind, and the parts of `ptr` for `null`.
    pub fn levels(self) -> (u8, Base)
    {
        match self
        {
            DataKind::Int(n) => (0, Base::Int(n)),
            DataKind::Float(bits) => (0, Base::Float(bits)),
            DataKind::Char => (0, Base::Char),
            DataKind::Bool => (0, Base::Bool),
            DataKind::Ptr(p) => (p.depth, p.base),
            DataKind::Null => (1, Base::Opaque),
            DataKind::List(l) => (0, Base::List(l)),
            DataKind::Struct(s) => (0, Base::Struct(s)),
            DataKind::Slice(s) => (0, Base::Slice(s)),
        }
    }

    /// The kind of an address of a value of this kind.
    ///
    /// # Returns
    ///
    /// The pointer kind one level deeper; `null` counts as `ptr`.
    pub fn pointer_to(self) -> DataKind
    {
        let (depth, base) = self.levels();
        let raw = self.ptr_kind().map_or(0, |p| p.raw);
        let depth = depth.saturating_add(1);
        DataKind::Ptr(PtrKind { raw, ..PtrKind::owner(depth, base) })
    }

    /// The kind of a raw address of a value of this kind.
    ///
    /// # Returns
    ///
    /// The pointer kind one level deeper, with the new level raw.
    pub fn raw_pointer_to(self) -> DataKind { self.pointer_to().with_outer_raw() }

    /// The kind a pointer of this kind points at.
    ///
    /// # Returns
    ///
    /// The kind one level shallower, without a borrow mark, or `None` for a
    /// kind that is no pointer, for `ptr` and for `null`.
    pub fn pointee(self) -> Option<DataKind>
    {
        let DataKind::Ptr(p) = self
        else
        {
            return None;
        };
        ret_if!(self.is_opaque(), None);
        let depth = p.depth.saturating_sub(1);
        ret_if!(depth == 0, DataKind::over(p.base, 0).into());
        let inner =
            PtrKind { raw: p.raw & !p.outer_bit(), ..PtrKind::owner(depth, p.base) };
        Some(DataKind::Ptr(inner))
    }

    /// Whether the kind is a pointer.
    ///
    /// # Returns
    ///
    /// `true` for a pointer kind, `ptr` included, but not for `null`.
    pub fn is_ptr(self) -> bool { matches!(self, DataKind::Ptr(_)) }

    /// Whether the kind is the opaque pointer `ptr`.
    ///
    /// # Returns
    ///
    /// `true` for `ptr` in each flavour: `ptr`, `&ptr` and `rawptr`.
    pub fn is_opaque(self) -> bool
    {
        matches!(self, DataKind::Ptr(p) if p.depth == 1 && p.base == Base::Opaque)
    }

    /// The pointer part of the kind.
    ///
    /// # Returns
    ///
    /// The levels, flavours and base, or `None` for a kind that is no
    /// pointer.
    pub fn ptr_kind(self) -> Option<PtrKind>
    {
        tern!(let DataKind::Ptr(p) = self, Some(p), None)
    }

    /// Whether the kind is a pointer whose outer level is an owner.
    ///
    /// # Returns
    ///
    /// `true` for `*i32` and `ptr`, but not for a borrow or a raw address.
    pub fn is_owner_ptr(self) -> bool { self.ptr_kind().is_some_and(PtrKind::is_owner) }

    /// Whether the kind is a borrow: a pointer whose outer level is a
    /// borrow, or a slice, which borrows its cells.
    ///
    /// # Returns
    ///
    /// `true` for `&*i32`, `&ptr` and `&[]i32`.
    pub fn is_borrow(self) -> bool
    {
        self.is_slice() || self.ptr_kind().is_some_and(PtrKind::is_borrowed)
    }

    /// Whether the kind is a pointer whose outer level is a raw address.
    ///
    /// # Returns
    ///
    /// `true` for `~i32` and `rawptr`.
    pub fn is_raw_ptr(self) -> bool
    {
        self.ptr_kind().is_some_and(PtrKind::is_raw_outer)
    }

    /// Whether the kind is a pointer the checker follows: an owner or a
    /// borrow at its outer level.
    ///
    /// # Returns
    ///
    /// `true` for `*i32`, `&*i32` and `ptr`, but not for `~i32`.
    pub fn is_checked_ptr(self) -> bool { self.is_ptr() && !self.is_raw_ptr() }

    /// Whether a value of the kind is moved, not copied, when it goes into
    /// another place.
    ///
    /// # Returns
    ///
    /// `true` for an owner and for a list whose cells are moved.
    pub fn is_move_kind(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.is_move_kind());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.is_move_kind()));
        self.is_owner_ptr()
    }

    /// Whether a value of the kind is duplicated when it goes into another
    /// place.
    ///
    /// # Returns
    ///
    /// `true` for each kind that is not a move kind.
    pub fn is_dup_kind(self) -> bool { !self.is_move_kind() }

    /// Whether a value of the kind is a borrow or has a borrow in it.
    ///
    /// # Returns
    ///
    /// `true` for a borrow and for a list whose cells hold one.
    pub fn holds_borrow(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_borrow());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_borrow()));
        self.is_borrow()
    }

    /// The kind with its outer address level turned into a borrow.
    ///
    /// # Returns
    ///
    /// The borrow of an owner; a raw address and each kind that is no
    /// pointer come back unchanged.
    pub fn as_borrowed(self) -> DataKind
    {
        tern!(let DataKind::Ptr(p) = self, DataKind::Ptr(p.as_borrowed()), self)
    }

    /// The kind with its outer address level turned into an owner.
    ///
    /// # Returns
    ///
    /// The kind without a borrow mark.
    pub fn as_owner(self) -> DataKind
    {
        tern!(let DataKind::Ptr(p) = self, DataKind::Ptr(p.as_owner()), self)
    }

    /// Whether two kinds describe blocks of the same shape, whether or not
    /// the borrow mark of their outer pointer level sets them apart.
    ///
    /// # Arguments
    ///
    /// * `other` - The kind to compare with.
    ///
    /// # Returns
    ///
    /// `true` when the kinds are equal up to the borrow mark of their
    /// outer pointer level; a raw level is part of the shape. A list, as a
    /// kind or as the base of a pointer, compares whole, the flavour of its
    /// cells included, so a list of borrows never has the shape of a list
    /// of owners.
    pub fn same_shape(self, other: DataKind) -> bool
    {
        let (Some(a), Some(b)) = (self.ptr_kind(), other.ptr_kind())
        else
        {
            return self == other;
        };
        a.shape() == b.shape()
    }

    /// Whether a value of the kind is an address.
    ///
    /// # Returns
    ///
    /// `true` for a pointer kind or `null`.
    pub fn is_address(self) -> bool { self.is_ptr() || self == DataKind::Null }

    /// The size of a value of the kind, in octets.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count over 8: 4 for a character, 1 for a boolean, the
    /// platform size for an address, and the sum of the cells for a list. A
    /// list too large for a `u32` gives `u32::MAX`; see
    /// [`DataKind::checked_size`].
    pub fn size(self, platform: u16) -> u32
    {
        self.checked_size(platform).unwrap_or(u32::MAX)
    }

    /// The size of a value of the kind, in octets, when it fits a `u32`.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The size, or `None` for a list too large for the machine. A slice is
    /// two words: its address and its count.
    pub fn checked_size(self, platform: u16) -> Option<u32>
    {
        ret_if!(let Some(s) = self.as_struct(), s.size(platform));
        ret_if!(self.is_slice(), Some(u32::from(platform) / 4));
        let Some(l) = self.as_list()
        else
        {
            return Some(u32::from(self.bits(platform)) / 8);
        };
        l.cell.checked_size(platform)?.checked_mul(l.len)
    }

    /// The alignment of a value of the kind, in octets.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The size, at most 16; for a list, the alignment of its cells.
    pub fn align(self, platform: u16) -> u32
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.align(platform));
        ret_if!(let Some(s) = self.as_struct(), s.align(platform));
        ret_if!(self.is_slice(), (u32::from(platform) / 8).min(16));
        self.size(platform).min(16)
    }

    /// The integer part of the kind.
    ///
    /// # Returns
    ///
    /// The sign and width, or `None` for a kind that is no integer.
    pub fn int_kind(self) -> Option<IntKind>
    {
        tern!(let DataKind::Int(n) = self, Some(n), None)
    }

    /// Whether the kind is an integer.
    ///
    /// # Returns
    ///
    /// `true` for an integer kind.
    pub fn is_int(self) -> bool { matches!(self, DataKind::Int(_)) }

    /// Whether the kind is an unsigned integer.
    ///
    /// # Returns
    ///
    /// `true` for an unsigned integer kind.
    pub fn is_unsigned(self) -> bool { self.int_kind().is_some_and(|n| !n.signed) }

    /// Whether the kind is a floating-point number.
    ///
    /// # Returns
    ///
    /// `true` for a floating-point kind.
    pub fn is_float(self) -> bool { matches!(self, DataKind::Float(_)) }

    /// Whether the kind is a number.
    ///
    /// # Returns
    ///
    /// `true` for an integer or a floating-point kind.
    pub fn is_numeric(self) -> bool { self.is_int() || self.is_float() }

    /// The bit count of the kind.
    ///
    /// # Arguments
    ///
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// The bit count; 32 for a character, 8 for a boolean, the platform
    /// size for an address. A list counts the bits of its cells, at most
    /// `u16::MAX`; use [`DataKind::size`] for lists.
    pub fn bits(self, platform: u16) -> u16
    {
        match self
        {
            DataKind::Int(n) => n.bits(platform),
            DataKind::Float(bits) => bits,
            DataKind::Char => 32,
            DataKind::Bool => 8,
            DataKind::Ptr(_) | DataKind::Null => platform,
            DataKind::List(_) | DataKind::Struct(_) | DataKind::Slice(_) =>
            {
                let bits = u64::from(self.size(platform)) * 8;
                u16::try_from(bits).unwrap_or(u16::MAX)
            }
        }
    }

    /// Whether a value of this kind converts to `target` on its own, with no
    /// loss of data.
    ///
    /// # Arguments
    ///
    /// * `target` - The kind the place expects.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// `true` for the same kind, a wider integer of the same sign, a wider
    /// signed integer from an unsigned one, a float whose fraction holds the
    /// integer, a wider float, `null` to a pointer, and a pointer to or from
    /// `ptr` of the same flavour: owner, borrow or raw. An owner or a borrow
    /// whose pointee holds an address needs a raw step to pass through
    /// `ptr`, as a cast does, see [`DataKind::reads_addresses_as`].
    pub fn widens_to(self, target: DataKind, platform: u16) -> bool
    {
        ret_true!(self == target);
        match (self, target)
        {
            (DataKind::Int(a), DataKind::Int(b)) => int_widens(a, b, platform),
            (DataKind::Int(a), DataKind::Float(b)) => a.bits(platform) <= precision(b),
            (DataKind::Float(a), DataKind::Float(b)) => a < b,
            (DataKind::Null, DataKind::Ptr(_)) => true,
            (DataKind::Ptr(_), DataKind::Ptr(_)) =>
            {
                self.meets_pointer(target) && !self.reads_addresses_as(target)
            }
            _ => false,
        }
    }

    /// Whether two pointers of one flavour address blocks of one shape, or
    /// one of them is `ptr`. This is enough to compare the two addresses;
    /// a value that goes into a place also needs
    /// [`DataKind::reads_addresses_as`] to be false.
    ///
    /// # Arguments
    ///
    /// * `other` - The other kind.
    ///
    /// # Returns
    ///
    /// `true` for two such pointer kinds.
    pub fn meets_pointer(self, other: DataKind) -> bool
    {
        let (Some(a), Some(b)) = (self.ptr_kind(), other.ptr_kind())
        else
        {
            return false;
        };
        let same_flavour =
            a.is_borrowed() == b.is_borrowed() && a.is_raw_outer() == b.is_raw_outer();
        let opaque = self.is_opaque() || other.is_opaque();
        same_flavour && (opaque || a.shape() == b.shape())
    }

    /// Whether an integer literal value fits this kind.
    ///
    /// # Arguments
    ///
    /// * `value` - The literal value.
    /// * `negated` - Whether a negate operator stands before the literal.
    /// * `platform` - Bit count of the platform-sized kinds.
    ///
    /// # Returns
    ///
    /// `true` when the kind is an integer whose range holds the value.
    pub fn fits_int(self, value: u128, negated: bool, platform: u16) -> bool
    {
        let Some(n) = self.int_kind()
        else
        {
            return false;
        };
        let bits = n.bits(platform);
        let max = u128::MAX >> (128 - bits);
        ret_if!(!n.signed, tern!(negated, value == 0, value <= max));
        let half = 1u128 << (bits - 1);
        tern!(negated, value <= half, value < half)
    }

    /// Whether a cast between two owners or borrows reads what the block
    /// holds as another kind, when one side holds an address there: it
    /// would read stored addresses as other data, or data as addresses,
    /// which the checker cannot follow. Such a cast goes through a raw
    /// address.
    ///
    /// # Arguments
    ///
    /// * `to` - The target kind.
    ///
    /// # Returns
    ///
    /// `true` when both kinds are owners or borrows, a pointee holds an
    /// address at some level, and the pointees differ in shape; an opaque
    /// pointee differs from each other one.
    pub fn reads_addresses_as(self, to: DataKind) -> bool
    {
        ret_false!(!self.is_checked_ptr() || !to.is_checked_ptr());
        let (a, b) = (self.pointee(), to.pointee());
        let holds = |k: Option<DataKind>| k.is_some_and(DataKind::holds_address);
        ret_false!(!holds(a) && !holds(b));
        let same = match (a, b)
        {
            (Some(a), Some(b)) => a.same_shape(b),
            (None, None) => true,
            _ => false,
        };
        !same
    }

    /// Whether a value of the kind holds an address at some level: a
    /// pointer, a slice, or a list whose cells hold one.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_address(self) -> bool
    {
        ret_true!(self.is_slice());
        ret_if!(let Some(list) = self.as_list(), list.cell.holds_address());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_address()));
        self.is_ptr()
    }

    /// Whether the conversion verb can turn this kind into `to`.
    ///
    /// # Arguments
    ///
    /// * `to` - The target kind.
    ///
    /// # Returns
    ///
    /// `true` between numbers, between characters and integers, from a
    /// boolean to an integer, between addresses, from an address to an
    /// integer, from an integer to a raw address, and for the same kind.
    /// An integer never becomes an owner or a borrow straight off: the
    /// raw address in between is the step a reader can search for. For the
    /// same reason a cast between owners and borrows keeps the shape of a
    /// pointee that holds an address, see [`DataKind::reads_addresses_as`].
    pub fn cast_allowed(self, to: DataKind) -> bool
    {
        ret_false!(self.is_aggregate() || to.is_aggregate());
        ret_true!(self == to);
        ret_true!(self.is_numeric() && to.is_numeric());
        ret_false!(self.reads_addresses_as(to));
        ret_true!(self.is_address() && (to.is_address() || to.is_int()));
        ret_true!(self.is_int() && to.is_address() && !to.is_checked_ptr());
        ret_false!(!to.is_int() && !self.is_int());
        matches!(
            (self, to),
            (DataKind::Char, DataKind::Int(_))
                | (DataKind::Int(_), DataKind::Char)
                | (DataKind::Bool, DataKind::Int(_))
        )
    }
}

/// The base a struct name stands for.
///
/// # Arguments
///
/// * `kind` - The kind phrase, with a [`BaseKind::Named`] base.
/// * `structs` - The kind of each declared struct.
///
/// # Returns
///
/// The struct, or `isize` for a name that is not declared.
fn named_base(kind: &Kind, structs: &StructTable) -> Base
{
    let found = kind.name.as_ref().and_then(|n| structs.get(&n.name)).copied();
    found.map_or(Base::Int(IntKind::new(true, Width::Platform)), |k| k.levels().1)
}

/// Read the short name of a plain kind such as `i8`, `usize`, `f32`,
/// `char`.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The kind, or `None` for an unknown name.
fn parse_plain(name: &str) -> Option<DataKind>
{
    let named = [("isize", DataKind::ISIZE), ("usize", DataKind::USIZE)];
    let named =
        named.into_iter().chain([("char", DataKind::Char), ("bool", DataKind::Bool)]);
    let found = named.into_iter().find(|(n, _)| *n == name).map(|(_, k)| k);
    ret_if!(found.is_some(), found);
    let (head, digits) = name.split_at_checked(1)?;
    ret_if!(!digits.bytes().all(|c| c.is_ascii_digit()), None);
    let bits: u16 = digits.parse().ok()?;
    let int = |signed| DataKind::Int(IntKind::new(signed, Width::Bits(bits)));
    match head
    {
        "i" | "u" => INT_BITS.contains(&bits).then(|| int(head == "i")),
        "f" => FLOAT_BITS.contains(&bits).then_some(DataKind::Float(bits)),
        _ => None,
    }
}

/// The short name of a pointer kind.
///
/// # Arguments
///
/// * `p` - The kind.
/// * `full` - Whether a struct base is written as its signature.
///
/// # Returns
///
/// A leading `&` for a borrow, then one mark per address level from the
/// outer one in, `*` for an owner and `~` for a raw level, then the base
/// name; `ptr` counts as one level itself and reads `rawptr` when raw.
fn ptr_name(p: PtrKind, full: bool) -> String
{
    let plain = p.base.scalar();
    let marked = usize::from(p.depth) - usize::from(plain.is_none());
    let mark = |at: usize| {
        let level = usize::from(p.depth) - 1 - at;
        tern!(p.raw.checked_shr(level as u32).unwrap_or(0) & 1 == 1, '~', '*')
    };
    let marks: String = (0..marked).map(mark).collect();
    let borrow = tern!(p.borrowed, "&", "");
    let Some(plain) = plain
    else
    {
        return format!("{borrow}{marks}{}", tern!(p.raw & 1 == 1, "rawptr", "ptr"));
    };
    format!("{borrow}{marks}{}", plain.spell(full))
}

/// Read the base part of a short name, past its address marks.
///
/// # Arguments
///
/// * `name` - The name with the leading `&`, `*` and `~` taken off.
///
/// # Returns
///
/// The base and whether it is a raw opaque address, or `None` for an unknown
/// name.
fn parse_base(name: &str) -> Option<(Base, bool)>
{
    ret_if!(name == "ptr", Some((Base::Opaque, false)));
    if let Some(cell) = name.strip_prefix("&[]")
    {
        return Some((DataKind::slice_of(DataKind::parse(cell)?).levels().1, false));
    }
    ret_if!(name.ends_with('}'), parse_sig(name).map(|s| (Base::Struct(s), false)));
    ret_if!(name == "rawptr", Some((Base::Opaque, true)));
    let Some(inner) = name.strip_prefix('[')
    else
    {
        return Some((parse_plain(name)?.levels().1, false));
    };
    let (count, cell) = inner.split_once(']')?;
    let len = tern!(count.is_empty(), 0, count.parse().ok()?);
    let list = DataKind::list_of(DataKind::parse(cell)?, len);
    Some((list.levels().1, false))
}

/// The short name of a list kind.
///
/// # Arguments
///
/// * `l` - The kind.
/// * `full` - Whether a struct cell is written as its signature.
///
/// # Returns
///
/// `[n]` then the cell name, such as `[3]isize`; `[]` while the count is
/// open.
fn list_name(l: &ListKind, full: bool) -> String
{
    let count = tern!(l.len == 0, String::new(), l.len.to_string());
    format!("[{count}]{}", l.cell.spell(full))
}

/// The short name of an integer kind.
///
/// # Arguments
///
/// * `n` - The kind.
///
/// # Returns
///
/// The name, such as `i8` or `usize`.
fn int_name(n: IntKind) -> String
{
    let sign = tern!(n.signed, "i", "u");
    match n.width
    {
        Width::Platform => format!("{sign}size"),
        Width::Bits(bits) => format!("{sign}{bits}"),
    }
}

/// Whether integer kind `a` converts to integer kind `b` on its own.
///
/// # Arguments
///
/// * `a` - The source kind.
/// * `b` - The target kind.
/// * `platform` - Bit count of the platform-sized kinds.
///
/// # Returns
///
/// `true` when `b` is wider and no sign is lost.
fn int_widens(a: IntKind, b: IntKind, platform: u16) -> bool
{
    let wider = a.bits(platform) < b.bits(platform);
    wider && !(a.signed && !b.signed)
}

/// Significand bits of a floating-point kind, the implicit bit included.
///
/// # Arguments
///
/// * `bits` - The bit count of the float kind.
///
/// # Returns
///
/// The number of integer bits the kind holds without loss.
fn precision(bits: u16) -> u16
{
    match bits
    {
        16 => 11,
        32 => 24,
        64 => 53,
        128 => 113,
        _ => 0,
    }
}

impl From<DataKind> for String
{
    /// The short name of the kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind.
    ///
    /// # Returns
    ///
    /// The name.
    fn from(kind: DataKind) -> String { kind.sig_name() }
}

impl TryFrom<String> for DataKind
{
    type Error = String;

    /// Read a short name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The kind, or an error for an unknown name.
    fn try_from(name: String) -> Result<Self, Self::Error>
    {
        DataKind::parse(&name).ok_or_else(|| format!("unknown data kind {name}"))
    }
}
