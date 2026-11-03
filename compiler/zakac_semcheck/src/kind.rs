use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

use serde::Deserialize;
use serde::Serialize;
use strum::IntoStaticStr;
use zakac_ast::kind::BaseKind;
use zakac_ast::kind::Cells;
use zakac_ast::kind::Kind;
use zakac_ast::kind::Wrap;
use zakac_rust_sugars::do_if;
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
    /// Owned text of a form.
    Text(TextForm),
    /// A bare `လိပ်စာ`, whose target kind is not known.
    Opaque,
    /// A list, for a pointer to a whole list.
    List(&'static ListKind),
    /// A struct, for a pointer to a whole struct.
    Struct(&'static StructKind),
    /// A slice, for a pointer to a slice value.
    Slice(&'static SliceKind),
    /// A plain or a heap list, for a pointer to one.
    Seq(&'static SeqKind),
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
            Base::Text(form) => Some(DataKind::Text(form)),
            Base::Opaque => None,
            Base::List(l) => Some(DataKind::List(l)),
            Base::Struct(s) => Some(DataKind::Struct(s)),
            Base::Slice(s) => Some(DataKind::Slice(s)),
            Base::Seq(s) => Some(DataKind::Seq(s)),
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
    /// Whether the cells are u8s that hold UTF-8 text: the view
    /// `စာသား တစ်စိတ်တစ်ပိုင်း`, which prints as text.
    pub text: bool,
}

/// Each slice kind made so far, so a kind is made once and remains `Copy`.
static SLICES: OnceLock<Mutex<HashSet<&'static SliceKind>>> = OnceLock::new();

/// A plain or a heap list kind: cells of one kind in a block of their own,
/// whose count is known at run time. A value is three words, laid out as
/// text is: the address of the first cell, the count, and the room of its
/// heap block in octets. A room of 0 means the list owns no block: it holds
/// no cells, or it reads cells it was lent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeqKind
{
    /// The kind of each cell.
    pub cell: DataKind,
    /// Whether it is a heap list (`စာရင်းရှင်`), which owns a block from its
    /// first cell on; else a plain list (`စာရင်း`), which takes one at its
    /// first edit.
    pub heap: bool,
}

/// Each plain and heap list kind made so far, so a kind is made once and
/// remains `Copy`.
static SEQS: OnceLock<Mutex<HashSet<&'static SeqKind>>> = OnceLock::new();

/// A pointer kind: `depth` levels of address over a base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtrKind
{
    /// The number of address levels, at least 1. An opaque base counts
    /// itself, so `ptr` has depth 1 and `ptr*` depth 2.
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
/// `i32*` or `ptr`.
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
    /// `စာသား`: owned UTF-8 text, three words: the address of its u8s,
    /// their count, and the room of its heap block. Room 0 means the u8s
    /// sit in static storage, so nothing frees them. The first two words
    /// are laid out as a slice's. The form tells whether the u8s can change
    /// and where the u8s live; the three forms share the same three words.
    Text(TextForm),
    /// A plain list (`ကိန်းပြည့် စာရင်း`) or a heap list (`ကိန်းပြည့်
    /// စာရင်းရှင်`): three words, see [`SeqKind`].
    Seq(&'static SeqKind),
}

/// The form of owned text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum TextForm
{
    /// `စာသား`: static until its first edit, then its own heap block.
    #[strum(serialize = "text")]
    Plain,
    /// `စာသားသေ`: static text, known at compile time; its u8s never change.
    #[strum(serialize = "textf")]
    Fixed,
    /// `စာသားရှင်`: its own heap block from the start.
    #[strum(serialize = "text~")]
    Heap,
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
    /// `စာသား`, text of the plain form.
    pub const TEXT: DataKind = DataKind::Text(TextForm::Plain);
    /// `လက္ခဏာမဲ့ ၈ ဘစ် ကိန်းပြည့်`, the cell of a text.
    pub const U8: DataKind = DataKind::Int(IntKind::new(false, Width::Bits(8)));

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
            BaseKind::Text => Base::Text(TextForm::Plain),
            BaseKind::FixedText => Base::Text(TextForm::Fixed),
            BaseKind::HeapText => Base::Text(TextForm::Heap),
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
            Wrap::List(Cells::Inferred) => DataKind::seq_of(inner, false),
            Wrap::List(Cells::Taken) => DataKind::list_of(inner, 0),
            Wrap::List(Cells::Heap) => DataKind::seq_of(inner, true),
            Wrap::Slice => DataKind::slice_of(inner),
            Wrap::View => DataKind::text_view(),
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
        DataKind::slice_kind(SliceKind { cell, text: false })
    }

    /// The view of text, `စာသား တစ်စိတ်တစ်ပိုင်း`: a slice of u8s that hold
    /// UTF-8.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn text_view() -> DataKind
    {
        DataKind::slice_kind(SliceKind { cell: DataKind::U8, text: true })
    }

    /// The slice kind with these parts, made once and kept for the rest of
    /// the run.
    ///
    /// # Arguments
    ///
    /// * `wanted` - The parts.
    ///
    /// # Returns
    ///
    /// The kind.
    fn slice_kind(wanted: SliceKind) -> DataKind
    {
        let slices = SLICES.get_or_init(Mutex::default);
        let mut slices = slices.lock().unwrap_or_else(PoisonError::into_inner);
        ret_if!(let Some(found) = slices.get(&wanted), DataKind::Slice(found));
        let made: &'static SliceKind = Box::leak(Box::new(wanted));
        slices.insert(made);
        DataKind::Slice(made)
    }

    /// The plain or heap list kind of cells of `cell`, made once and kept
    /// for the rest of the run.
    ///
    /// # Arguments
    ///
    /// * `cell` - The kind of each cell.
    /// * `heap` - Whether it is a heap list.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn seq_of(cell: DataKind, heap: bool) -> DataKind
    {
        let wanted = SeqKind { cell, heap };
        let seqs = SEQS.get_or_init(Mutex::default);
        let mut seqs = seqs.lock().unwrap_or_else(PoisonError::into_inner);
        ret_if!(let Some(found) = seqs.get(&wanted), DataKind::Seq(found));
        let made: &'static SeqKind = Box::leak(Box::new(wanted));
        seqs.insert(made);
        DataKind::Seq(made)
    }

    /// The plain or heap list part of the kind.
    ///
    /// # Returns
    ///
    /// The parts, or `None` for a kind that is no plain or heap list.
    pub fn as_seq(self) -> Option<&'static SeqKind>
    {
        tern!(let DataKind::Seq(s) = self, Some(s), None)
    }

    /// Whether the kind is a plain or a heap list.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn is_seq(self) -> bool { matches!(self, DataKind::Seq(_)) }

    /// The kind of each cell of a list of each form or of a slice.
    ///
    /// # Returns
    ///
    /// The cell kind, or `None` for a kind with no cells.
    pub fn cell_kind(self) -> Option<DataKind>
    {
        ret_if!(let Some(l) = self.as_list(), Some(l.cell));
        ret_if!(let Some(s) = self.as_seq(), Some(s.cell));
        self.as_slice().map(|s| s.cell)
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

    /// Whether the kind is the view of text.
    ///
    /// # Returns
    ///
    /// `true` for `စာသား တစ်စိတ်တစ်ပိုင်း`.
    pub fn is_text_view(self) -> bool { self.as_slice().is_some_and(|s| s.text) }

    /// Whether the kind is text: owned, or a view.
    ///
    /// # Returns
    ///
    /// `true` for `စာသား` and `စာသား တစ်စိတ်တစ်ပိုင်း`.
    pub fn is_textual(self) -> bool { self.is_text() || self.is_text_view() }

    /// Whether the kind is owned text, of one of its three forms.
    ///
    /// # Returns
    ///
    /// `true` for `စာသား`, `စာသားသေ` and `စာသားရှင်`.
    pub fn is_text(self) -> bool { matches!(self, DataKind::Text(_)) }

    /// Whether a value of the kind holds heap text, which owns a heap block
    /// from the start: the text itself, or a list or struct with it inside.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_heap_text(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_heap_text());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_heap_text()));
        self == DataKind::Text(TextForm::Heap) || self.as_seq().is_some_and(|s| s.heap)
    }

    /// Whether a value of the kind holds owned text, which is dropped when
    /// its place ends: text itself, or a list or struct with text in it.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_text(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_text());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_text()));
        self.is_text() || self.is_seq()
    }

    /// Whether a value of the kind can own a heap block that its place frees
    /// as it ends: plain or heap text, a plain or heap list, or a fixed list
    /// or a struct with one inside. Fixed text is static and owns none.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn needs_drop(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.needs_drop());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.needs_drop()));
        (self.is_text() && self != DataKind::Text(TextForm::Fixed)) || self.is_seq()
    }

    /// Whether a value of the kind holds a plain or a heap list: the list
    /// itself, or a fixed list or a struct with one inside.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_seq(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_seq());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_seq()));
        self.is_seq()
    }

    /// The kind with each plain or heap list turned into a fixed list whose
    /// count is still open, at each depth of lists: the kind of a constant.
    ///
    /// # Returns
    ///
    /// The kind.
    pub fn fixed_form(self) -> DataKind
    {
        ret_if!(let Some(s) = self.as_seq(), DataKind::list_of(s.cell.fixed_form(), 0));
        ret_if!(let Some(l) = self.as_list(), DataKind::list_of(l.cell.fixed_form(), l.len));
        self
    }

    /// Whether a value of the kind holds an owner pointer, whose block the
    /// program must free or move on: the pointer itself, or a list or struct
    /// with one in it.
    ///
    /// # Returns
    ///
    /// `true` for such a kind.
    pub fn holds_owner_ptr(self) -> bool
    {
        ret_if!(let Some(l) = self.as_list(), l.cell.holds_owner_ptr());
        ret_if!(let Some(l) = self.as_seq(), l.cell.holds_owner_ptr());
        ret_if!(let Some(s) = self.as_struct(), s.fields().iter().any(|(_, k)| k.holds_owner_ptr()));
        self.is_owner_ptr()
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
        self.is_list()
            || self.is_struct()
            || self.is_slice()
            || self.is_text()
            || self.is_seq()
    }

    /// Whether some list level of the kind has no cell count so far.
    ///
    /// # Returns
    ///
    /// `true` when a count is still 0.
    pub fn has_open_count(self) -> bool
    {
        ret_if!(let Some(s) = self.as_slice(), s.cell.has_open_count());
        ret_if!(let Some(s) = self.as_seq(), s.cell.has_open_count());
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
        if let (Some(want), Some(have)) = (self.as_seq(), found.cell_kind())
        {
            return Some(DataKind::seq_of(want.cell.fill_counts(have)?, want.heap));
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

    /// Read a short name such as `i8`, `usize`, `char`, `i32*`, `i32&`,
    /// `i32^`, `ptr`, `ptr&`, `rawptr`, `isize[3]`, `isize[][..]`, `text[..]`
    /// or `null`: a base, then one mark for each level in the order of the
    /// Burmese words.
    ///
    /// # Arguments
    ///
    /// * `name` - The name.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for an unknown name, a borrowed raw level, a
    /// level over a borrow, and a slice of no list.
    pub fn parse(name: &str) -> Option<DataKind>
    {
        ret_if!(name == "null", Some(DataKind::Null));
        let (base, marks) = split_marks(name);
        let base = parse_base(base)?;
        DataKind::wrap_marks(base, marks)
    }

    /// Wrap a kind in the levels its marks name, from left to right: `*` an
    /// owner, `^` a raw address, `&` a borrow, `[n]` a list of `n` cells,
    /// `[]` a list whose count is open, and `[..]` a slice of the list or
    /// the view of the text before it. On `ptr`, which is an address
    /// itself, `&` borrows it and adds no level.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind under the marks.
    /// * `marks` - The marks.
    ///
    /// # Returns
    ///
    /// The kind, or `None` for a mark that does not read.
    pub fn wrap_marks(kind: DataKind, marks: &str) -> Option<DataKind>
    {
        let mut kind = kind;
        let mut rest = marks;
        while let Some(ch) = rest.chars().next()
        {
            let len = tern!(ch == '[', rest.find(']')? + 1, ch.len_utf8());
            kind = wrap_mark(kind, &rest[..len])?;
            rest = &rest[len..];
        }
        Some(kind)
    }

    /// The short name, for messages.
    ///
    /// # Returns
    ///
    /// The name, such as `i8`, `usize`, `f64`, `char`, `bool`, `i32*`,
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
            DataKind::Slice(s) if s.text => "text[..]".to_string(),
            DataKind::Slice(s) => format!("{}[][..]", s.cell.spell(full)),
            DataKind::Seq(s) =>
            {
                format!("{}[]{}", s.cell.spell(full), tern!(s.heap, "~", ""))
            }
            DataKind::Text(form) => <&str>::from(form).to_string(),
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
            DataKind::Seq(s) => (0, Base::Seq(s)),
            DataKind::Text(form) => (0, Base::Text(form)),
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
    /// `true` for `ptr` in each flavour: `ptr`, `ptr&` and `rawptr`.
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
    /// `true` for `i32*` and `ptr`, but not for a borrow or a raw address.
    pub fn is_owner_ptr(self) -> bool { self.ptr_kind().is_some_and(PtrKind::is_owner) }

    /// Whether the kind is a borrow: a pointer whose outer level is a
    /// borrow, or a slice, which borrows its cells.
    ///
    /// # Returns
    ///
    /// `true` for `i32&`, `ptr&` and `i32[][..]`.
    pub fn is_borrow(self) -> bool
    {
        self.is_slice() || self.ptr_kind().is_some_and(PtrKind::is_borrowed)
    }

    /// Whether the kind is a pointer whose outer level is a raw address.
    ///
    /// # Returns
    ///
    /// `true` for `i32^` and `rawptr`.
    pub fn is_raw_ptr(self) -> bool
    {
        self.ptr_kind().is_some_and(PtrKind::is_raw_outer)
    }

    /// Whether the kind is a pointer the checker follows: an owner or a
    /// borrow at its outer level.
    ///
    /// # Returns
    ///
    /// `true` for `i32*`, `i32&` and `ptr`, but not for `i32^`.
    pub fn is_checked_ptr(self) -> bool { self.is_ptr() && !self.is_raw_ptr() }

    /// Whether a value of the kind is moved, not copied, when it goes into
    /// another place: a kind that holds an owner pointer, since a duplicate
    /// would free one block twice. Text and each other value are copied.
    ///
    /// # Returns
    ///
    /// `true` for an owner, and a list or struct with one in it.
    pub fn is_move_kind(self) -> bool { self.holds_owner_ptr() }

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
        ret_if!(let Some(l) = self.as_seq(), l.cell.holds_borrow());
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
        ret_if!(self.is_text() || self.is_seq(), Some(u32::from(platform) / 8 * 3));
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
        let words = self.is_slice() || self.is_text() || self.is_seq();
        ret_if!(words, (u32::from(platform) / 8).min(16));
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
            DataKind::List(_)
            | DataKind::Struct(_)
            | DataKind::Slice(_)
            | DataKind::Seq(_)
            | DataKind::Text(_) =>
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
            (DataKind::Text(_), DataKind::Text(_)) => true,
            (DataKind::List(a), DataKind::Seq(b)) => a.len > 0 && a.cell == b.cell,
            (DataKind::Seq(a), DataKind::Seq(b)) => a.cell == b.cell,
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
        ret_true!(self.is_slice() || self.is_text() || self.is_seq());
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
        let bytes = self.as_slice().is_some_and(|s| !s.text && s.cell == DataKind::U8);
        ret_true!(bytes && to.is_text_view());
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
    let named = named.into_iter().chain([
        ("char", DataKind::Char),
        ("bool", DataKind::Bool),
        ("text", DataKind::TEXT),
        ("textf", DataKind::Text(TextForm::Fixed)),
        ("text~", DataKind::Text(TextForm::Heap)),
    ]);
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
/// The base name, then one mark per address level from the base out: `*`
/// for an owner and `^` for a raw level; a borrowed outer level is `&` in
/// place of its `*`. `ptr` is a level itself and reads `rawptr` when raw;
/// a borrow of it adds `&` after its marks.
fn ptr_name(p: PtrKind, full: bool) -> String
{
    let plain = p.base.scalar();
    let first = usize::from(plain.is_none());
    let raw = |level: usize| p.raw.checked_shr(level as u32).unwrap_or(0) & 1 == 1;
    let levels = first..usize::from(p.depth);
    let mut marks: String = levels.map(|l| tern!(raw(l), '^', '*')).collect();
    let replace = p.borrowed && plain.is_some();
    do_if!(replace, _ = marks.pop());
    do_if!(p.borrowed, marks.push('&'));
    let base = plain
        .map_or_else(|| tern!(raw(0), "rawptr", "ptr").to_string(), |k| k.spell(full));
    format!("{base}{marks}")
}

/// Read the base of a short name: a plain kind, `ptr`, `rawptr` or a struct
/// signature.
///
/// # Arguments
///
/// * `name` - The base, with its marks taken off.
///
/// # Returns
///
/// The kind, or `None` for an unknown name.
fn parse_base(name: &str) -> Option<DataKind>
{
    ret_if!(name == "ptr", Some(DataKind::OPAQUE));
    ret_if!(name == "rawptr", Some(DataKind::OPAQUE.with_outer_raw()));
    ret_if!(name.ends_with('}'), parse_sig(name).map(DataKind::Struct));
    parse_plain(name)
}

/// Wrap a kind in the level one mark names.
///
/// # Arguments
///
/// * `kind` - The kind under the mark.
/// * `mark` - `*`, `^`, `&`, `[n]`, `[]` or `[..]`.
///
/// # Returns
///
/// The kind, or `None` for a mark that does not read here.
fn wrap_mark(kind: DataKind, mark: &str) -> Option<DataKind>
{
    // An address level over a borrow would lose the borrow, so no name
    // writes one; a list of borrows keeps them.
    let level = matches!(mark, "*" | "^" | "&");
    ret_if!(level && kind.is_borrow() && kind.is_ptr(), None);
    let opaque = kind.ptr_kind().is_some_and(|p| p.base == Base::Opaque);
    match mark
    {
        "*" => Some(kind.pointer_to()),
        "^" => Some(kind.raw_pointer_to()),
        "&" if opaque && !kind.is_raw_ptr() => Some(kind.as_borrowed()),
        "&" => Some(kind.pointer_to().as_borrowed()),
        "[..]" if kind.is_text() => Some(DataKind::text_view()),
        "[..]" => kind.as_seq().filter(|s| !s.heap).map(|s| DataKind::slice_of(s.cell)),
        "[]" => Some(DataKind::seq_of(kind, false)),
        "~" =>
        {
            kind.as_seq().filter(|s| !s.heap).map(|s| DataKind::seq_of(s.cell, true))
        }
        "[_]" => Some(DataKind::list_of(kind, 0)),
        _ =>
        {
            let count = mark.strip_prefix('[')?.strip_suffix(']')?;
            let len = tern!(count.is_empty(), 0, count.parse().ok()?);
            Some(DataKind::list_of(kind, len))
        }
    }
}

/// Split a short name into its base and its marks. The base ends at the
/// first `*`, `^`, `&` or `[` outside the braces of a struct signature.
///
/// # Arguments
///
/// * `name` - The name.
///
/// # Returns
///
/// The base and the marks after it.
pub fn split_marks(name: &str) -> (&str, &str)
{
    let mut depth = 0usize;
    for (at, ch) in name.char_indices()
    {
        match ch
        {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            '*' | '^' | '&' | '[' if depth == 0 => return name.split_at(at),
            _ => (),
        }
    }
    (name, "")
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
/// The cell name then `[n]`, such as `isize[3]`; `[]` while the count is
/// open.
fn list_name(l: &ListKind, full: bool) -> String
{
    let count = tern!(l.len == 0, "_".to_string(), l.len.to_string());
    format!("{}[{count}]", l.cell.spell(full))
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
